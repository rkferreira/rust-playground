use aws_config as aws;
use aws_sdk_route53 as aws_route53;
use aws_sdk_s3 as aws_s3;
use tokio;
pub extern crate aws_route53_backup_cli;

const MAX_DOMAIN_THREADS: usize = 2;

#[::tokio::main]
async fn main() {
    setup_tracing();

    let config      = aws::from_env().region("us-east-1").load().await;
    let client_r53  = aws_route53::Client::new(&config);
    let client_s3   = aws_s3::Client::new(&config);
    let domains     = aws_route53_backup_cli::list_domains(client_r53.clone()).await;

    let domains_len = domains.len();

    let mut tasks_domain = Vec::with_capacity(MAX_DOMAIN_THREADS);
    let mut tasks_writer = Vec::with_capacity(domains_len);

    let mut i = 0;
    while i < domains_len {
        for _thread in 0..MAX_DOMAIN_THREADS {
            if i < domains_len {
                let client_r53 = client_r53.clone();
                let domain = domains[i].clone();
                let name   = domain.name.clone();
                tasks_domain.push(
                    tokio::spawn(async move {
                        let output = aws_route53_backup_cli::list_resource_records(client_r53, domain).await;
                        (name, output)
                    })
                );
                i += 1;
            }
        }
        for task in tasks_domain.drain(..) {
            let (domain_name, record_set) = task.await.unwrap();
            //println!("output: {} \n {:?}", domain_name, record_set);
            let client_s3 = client_s3.clone();
            tasks_writer.push(
                tokio::spawn(async move {
                    aws_route53_backup_cli::format_and_write(client_s3, domain_name, record_set).await;
                })
            );
        }
    }
    for task in tasks_writer {
        task.await.unwrap();
    }
}

fn setup_tracing () {
    tracing_subscriber::fmt()
        .compact()
        .with_max_level(tracing::Level::INFO)
        .with_thread_ids(true)
        .with_target(false)
        .init();
}

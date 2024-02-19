use aws_config as aws;
use std::collections::HashMap;
use std::env;
pub extern crate list_aws_sso_permissions;


#[::tokio::main]
async fn main() {
    let config = aws::from_env().region("us-east-1").load().await;

    // global result
    let mut result = list_aws_sso_permissions::Accounts { dictionary: HashMap::new() };

    // get account number from command line
    let account = env::args().nth(1).unwrap_or("".to_string());

    // if no account number is provided, list all accounts
    if account.is_empty() {
        let _ = list_aws_sso_permissions::list_accounts(&config, &mut result).await;
    } else {
        result.dictionary.insert(account, HashMap::new());
    }
    let _ = list_aws_sso_permissions::list_permission_sets(&config, &mut result).await;
    let _ = list_aws_sso_permissions::list_assignments(&config, &mut result).await;

    println!("{:#?}", result);
}

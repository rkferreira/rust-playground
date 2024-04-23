use octocrab::{Octocrab, OctocrabBuilder};
use std::env;
use http::header::HeaderName;
use tokio;


#[tokio::main]
async fn main() -> () {
    let token = env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN not set");
    let octocrab = OctocrabBuilder::default()
        .personal_token(token)
        .build()
        .expect("Failed to create Octocrab instance");

    let mut page = octocrab
        .search()
        .repositories("org:dummy language:hcl")
        .per_page(100)
        .send()
        .await
        .expect("Failed to search repositories");

    let mut results = page.take_items();

    while let Ok(Some(mut new_page)) = octocrab.get_page(&page.next).await {
        results.append(&mut new_page.take_items());
        page = new_page;
    }

    for r in results {
        println!("https://github.com/{}", r.full_name.unwrap());
    }

    /*
    let mut code_results = Vec::new();

    for r in results {
        println!("{:?}", r.full_name);
        let repo = r.full_name.unwrap();
        let search = format!("repo:{} github.com:", repo);
        println!("{:?}", search);
        let mut page = octocrab
            .search()
            .code(&search)
            .per_page(10)
            .send()
            .await
            .expect("Failed to search code");

        //println!("{:?}", page);
        code_results.append(&mut page.take_items());

        while let Ok(Some(mut new_page)) = octocrab.get_page(&page.next).await {
            code_results.append(&mut new_page.take_items());
            page = new_page;
        }
    }
    */
}

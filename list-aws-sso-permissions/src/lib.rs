use aws_sdk_ssoadmin as ssoadmin;
use aws_sdk_organizations as organizations;
use aws_sdk_identitystore as identitystore;
use std::collections::HashMap;

// Instance ARN for SSO
pub const SSO_INSTANCE_ARN: &str = "arn:aws:sso:::instance/ssoins-xxx";

// Identity store
pub const IDENTITY_STORE: &str   = "d-xxx";


// Struct for holding results
// format:
//   account_id => permissions_set_name => permission_set_arn => [ GroupName/UserName ]
//
// sample:
//   "123456789012" => "AdministratorAccess" => "arn:aws:sso:::permissionSet/ssoins-xxx/ps-xxx" =>  [ "Admins" ]
//
#[derive(Debug)]
pub struct Accounts {
    pub dictionary: HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>
}

// Function to list all organization accounts
// and store them in a dictionary
pub async fn list_accounts(config: &aws_config::SdkConfig, result: &mut Accounts) -> Result<(), organizations::Error> {
    println!("Listing organization accounts");
    let client = organizations::Client::new(config);
    let mut response = client.list_accounts().send().await.unwrap();
    let mut account_ids = get_accounts_id(&response).await;
    while let Some(ref next_token) = response.next_token {
        response = client.list_accounts().next_token(next_token).send().await.unwrap();
        account_ids.append(&mut get_accounts_id(&response).await);
    }
    account_ids.sort();
    for account_id in account_ids {
        println!("Account: {:?}", account_id);
        result.dictionary.insert(account_id, HashMap::new());
    }
    println!("Organization accounts listed");
    Ok(())
}

// Function to get account ids from the list_accounts response
async fn get_accounts_id(accounts: &organizations::operation::list_accounts::ListAccountsOutput) -> Vec<String> {
    let mut account_ids: Vec<String> = Vec::new();
    for account in accounts.accounts.as_ref().unwrap() {
        account_ids.push(account.id.as_ref().unwrap().to_string());
    }
    account_ids
}

// Function to list all permission sets tied to the accounts
pub async fn list_permission_sets(config: &aws_config::SdkConfig, result: &mut Accounts) -> Result<(), ssoadmin::Error> {
    println!("Listing permission sets");
    let client = ssoadmin::Client::new(config);
    let account_list: Vec<String> = result.dictionary.keys().cloned().collect();

    for account_id in account_list {
        println!("Account: {:?}", account_id);
        let mut response = client.list_permission_sets_provisioned_to_account().instance_arn(SSO_INSTANCE_ARN).account_id(account_id.clone()).send().await.unwrap();
        let mut permission_sets: HashMap<String, HashMap<String, Vec<String>>> = get_permission_sets(&config, &response).await;
        while let Some(ref next_token) = response.next_token {
            response = client.list_permission_sets_provisioned_to_account().instance_arn(SSO_INSTANCE_ARN).account_id(account_id.clone()).next_token(next_token).send().await.unwrap();
            permission_sets.extend(get_permission_sets(&config, &response).await);
        }
        result.dictionary.get_mut(&account_id).unwrap().extend(permission_sets);
    }
    println!("Permission sets listed");
    Ok(())
}

// Function to get permission sets friendly names and arns
async fn get_permission_sets(config: &aws_config::SdkConfig, permissions: &ssoadmin::operation::list_permission_sets_provisioned_to_account::ListPermissionSetsProvisionedToAccountOutput) ->
HashMap<String, HashMap<String, Vec<String>>> {
    let client = ssoadmin::Client::new(config);
    let mut permission_sets: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();

    for permission_set in permissions.permission_sets.as_ref().unwrap() {
        let i = client.describe_permission_set().instance_arn(SSO_INSTANCE_ARN).permission_set_arn(permission_set).send().await.unwrap();
        let name = i.permission_set.unwrap().name.unwrap().to_string();
        permission_sets.insert(name.clone(), HashMap::new());
        permission_sets.get_mut(&name).unwrap().insert(permission_set.to_string(), Vec::new());
        println!("Permission_set: {:?}", permission_set);
    }
    permission_sets
}

// Function to list all account assignments
pub async fn list_assignments(config: &aws_config::SdkConfig, result: &mut Accounts) -> Result<(), ssoadmin::Error> {
    println!("Listing account assignments");
    let client = ssoadmin::Client::new(config);
    let account_list: Vec<String> = result.dictionary.keys().cloned().collect();

    for account_id in account_list {
        let permission_set_list: Vec<String> = result.dictionary.get(&account_id).unwrap().keys().cloned().collect();
        for permission_set in permission_set_list {
            let permission_set_arn = result.dictionary.get(&account_id).unwrap().get(&permission_set).unwrap().keys().next().unwrap().to_string();
            let mut response = client.list_account_assignments().instance_arn(SSO_INSTANCE_ARN).account_id(account_id.clone()).permission_set_arn(permission_set_arn.clone()).send().await.unwrap();
            let mut groups = get_groups_from_account(config, &response).await;
            while let Some(ref next_token) = response.next_token {
                response = client.list_account_assignments().instance_arn(SSO_INSTANCE_ARN).account_id(account_id.clone()).permission_set_arn(permission_set_arn.clone()).next_token(next_token).send().await.unwrap();
                groups.append(&mut get_groups_from_account(config, &response).await);
            }
            result.dictionary.get_mut(&account_id).unwrap().get_mut(&permission_set).unwrap().insert(permission_set_arn, groups);
        }
    }
    println!("Account assignments listed");
    Ok(())
}

// Function to get groups from account (principal ids)
async fn get_groups_from_account(config: &aws_config::SdkConfig, account: &ssoadmin::operation::list_account_assignments::ListAccountAssignmentsOutput) -> Vec<String> {
    let mut groups: Vec<String> = Vec::new();
    for assignment in account.account_assignments.as_ref().unwrap() {
        let display_name = get_user_group_name(config, &assignment.principal_id.as_ref().unwrap()).await;
        groups.push(display_name);
        //groups.push(assignment.principal_id.as_ref().unwrap().to_string());
    }
    groups
}

// Function to get display name for the principal id
async fn get_user_group_name(config: &aws_config::SdkConfig, group_id: &str) -> String {
    let client = identitystore::Client::new(config);
    let response = client.describe_group().identity_store_id(IDENTITY_STORE).group_id(group_id.to_string()).send().await;
    if let Ok(response) = response {
        return response.display_name.unwrap().to_string();
    } else {
        let response = client.describe_user().identity_store_id(IDENTITY_STORE).user_id(group_id.to_string()).send().await;
        if let Ok(response) = response {
            return response.display_name.unwrap().to_string();
        }
        return group_id.to_string();
    }
}

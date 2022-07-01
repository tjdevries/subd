use anyhow::Result;
use graphql_client::{GraphQLQuery, Response};
use hyper::header::{HeaderValue, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use subd_types::GithubUser;

// The paths are relative to the directory where your `Cargo.toml` is located.
// Both json and the GraphQL schema language are supported as sources for the schema
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "gql/gh.schema.graphql",
    query_path = "gql/is_sponsoring.query.graphql",
    response_derives = "Debug"
)]
pub struct IsSponsoring;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "gql/gh.schema.graphql",
    query_path = "gql/get_user.query.graphql",
    response_derives = "Debug,Clone"
)]
pub struct GetUser;

pub async fn get_github_user(login: &str) -> Result<Option<GithubUser>> {
    let gh_token = String::from("token ")
        + &std::env::var("GITHUB_ACCESS").expect("Should have GITHUB_ACCESS token");

    let request_body = GetUser::build_query(get_user::Variables {
        login: login.to_string(),
    });

    let client = reqwest::Client::new();
    let res = client
        .post("https://api.github.com/graphql")
        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .header(USER_AGENT, HeaderValue::from_static("subd"))
        .header(AUTHORIZATION, HeaderValue::from_str(&gh_token).unwrap())
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<get_user::ResponseData> = res.json().await?;
    if let None = response_body.data {
        return Ok(None);
    }

    let result = match response_body.data.unwrap().user {
        Some(result) => result,
        None => return Ok(None),
    };

    Ok(Some(GithubUser {
        id: result.id,
        login: result.login,
        name: match result.name {
            Some(name) => name,
            None => "UNKNOWN NAME".to_string(),
        },
    }))
}

pub async fn is_user_sponsoring(github_user: &str) -> Result<bool> {
    // TODO: Could probably make this static
    let gh_token = String::from("token ")
        + &std::env::var("GITHUB_ACCESS").expect("Should have GITHUB_ACCESS token");

    let request_body = IsSponsoring::build_query(is_sponsoring::Variables {
        name: github_user.to_string(),
    });

    let client = reqwest::Client::new();
    let res = client
        .post("https://api.github.com/graphql")
        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .header(USER_AGENT, HeaderValue::from_static("subd"))
        .header(AUTHORIZATION, HeaderValue::from_str(&gh_token).unwrap())
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<is_sponsoring::ResponseData> = res.json().await?;
    if let Some(errors) = response_body.errors {
        println!("ERRORS: {:?}", errors);
        return Ok(false);
    }

    Ok(response_body.data.unwrap().user.unwrap().is_sponsored_by)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

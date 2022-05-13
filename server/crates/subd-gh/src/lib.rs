use graphql_client::{GraphQLQuery, Response};

// The paths are relative to the directory where your `Cargo.toml` is located.
// Both json and the GraphQL schema language are supported as sources for the schema
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "gql/gh.schema.graphql",
    query_path = "gql/is_sponsoring.query.graphql",
    response_derives = "Debug"
)]
pub struct IsSponsoring;

pub async fn is_user_sponsoring(github_user: &str) -> anyhow::Result<bool> {
    let request_body = IsSponsoring::build_query(is_sponsoring::Variables {
        name: github_user.to_string(),
    });

    let client = reqwest::Client::new();
    let res = client
        .post("https://api.github.com/graphql")
        // .body(&request_body)
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<is_sponsoring::ResponseData> = res.json().await?;
    println!("{:#?}", response_body);

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

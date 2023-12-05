use reqwest::Client as ReqwestClient;
// use twitch_api2::{
//     helix::points::{
//         CreateCustomRewardBody, CreateCustomRewardRequest,
//         GetCustomRewardRequest, UpdateCustomRewardBody,
//         UpdateCustomRewardRequest,
//     },
//     twitch_oauth2::UserToken,
//     HelixClient,
// };

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Running rust stuff");

    // let helix: HelixClient<ReqwestClient> = HelixClient::default();
    //
    // let reqwest_client = helix.clone_client();
    // let token = UserToken::from_existing(
    //     &reqwest_client,
    //     subd_types::consts::get_twitch_broadcaster_oauth(),
    //     subd_types::consts::get_twitch_broadcaster_refresh(),
    //     None, // Client Secret
    // )
    // .await
    // .unwrap();
    //
    // let create = false;
    // let update = true;
    // if create {
    //     let req = CreateCustomRewardRequest::builder()
    //         .broadcaster_id(token.user_id.clone())
    //         .build();
    //
    //     let body = CreateCustomRewardBody::builder()
    //         .title("Created from cli")
    //         .cost(1000000)
    //         .build();
    //
    //     let sent = helix
    //         .req_post(req, body, &token)
    //         .await
    //         .expect("created reward");
    //     dbg!(sent);
    // } else if update {
    //     let id = "74447400-1357-4623-b458-5cff7fa8af67";
    //     let req = UpdateCustomRewardRequest::builder()
    //         .broadcaster_id(token.user_id.clone())
    //         .id(id)
    //         .build();
    //
    //     let body = UpdateCustomRewardBody::builder()
    //         .is_enabled(Some(true))
    //         .build();
    //
    //     let to_send =
    //         helix.req_patch(req, body, &token).await.expect("sent good");
    //     dbg!(to_send);
    // } else {
    //     let req = GetCustomRewardRequest::builder()
    //         .broadcaster_id(token.user_id.clone())
    //         .build();
    //
    //     let response = helix.req_get(req, &token).await.expect("yayayaya");
    //     dbg!(response);
    // }

    Ok(())
}

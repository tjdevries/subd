use anyhow::Result;
use twitter_v2::authorization::BearerToken;
use twitter_v2::id::NumericId;
// use twitter_v2::api_result::ApiResponse;
// use twitter_v2::authorization::{BearerToken, Oauth2Token};
// use twitter_v2::query::{
//     SpaceExpansion, SpaceField, TopicField, UserField,
// };
// use twitter_v2::Space;
// use twitter_v2::TwitterApi;

// |          --------- ^^^^^^^^^^ the trait `IntoNumericId` is not implemented for `&str`

#[tokio::main]
async fn main() -> Result<()> {
    let _otherside_guild_id = NumericId::new(1521585633445122048);
    // let other_guild_tweet = 1607890390811840512;
    let _phil_tweet = NumericId::new(1608588236452167681);
    let _phil_id = NumericId::new(34440817);

    let _auth =
        BearerToken::new(std::env::var("TWITTER_APP_BEARER_TOKEN").unwrap());
    Ok(())

    // let tweet = TwitterApi::new(auth)
    //     .get_tweet(phil_tweet)
    //     .tweet_fields([TweetField::AuthorId, TweetField::CreatedAt])
    //     .send()
    //     .await?
    //     .into_data()
    //     .expect("this tweet should exist");
    // println!("{:?}", tweet);

    // let auth =
    //     BearerToken::new(std::env::var("TWITTER_APP_BEARER_TOKEN").unwrap());

    // So this ID doesn't seem right
    // We need to figure out a different one
    // println!("{:?}", space);
    //
    // let auth =
    //     BearerToken::new(std::env::var("TWITTER_APP_BEARER_TOKEN").unwrap());

    // /(invited_user_ids, speaker_ids, creator_id, host_ids, topics_ids)/ expansions
    // 1ypKddPokoaKW
    // let space = TwitterApi::new(auth)
    //     .get_spaces_by_creator_ids([otherside_guild_id])
    //     // So these aren't working how I expected
    //     // I thought doing this could show the topics
    //     .topic_fields([
    //         TopicField::Id,
    //         TopicField::Name,
    //         TopicField::Description,
    //     ])
    //     .user_fields([
    //         UserField::CreatedAt,
    //         UserField::Description,
    //         UserField::Id,
    //         UserField::Name,
    //         // UserField::PinnedTweetId,
    //         // UserField::ProfileImageUrl,
    //         // UserField::PublicMetrics,
    //         UserField::Username,
    //         UserField::Verified,
    //         // Entities,
    //         // Location,
    //         // Protected,
    //         // Url,
    //         // Withheld,
    //     ])
    //     .expansions([
    //         SpaceExpansion::HostIds,
    //         // SpaceExpansion::InvitedUserIds,
    //         SpaceExpansion::SpeakerIds,
    //         SpaceExpansion::CreatorId,
    //     ])
    //     .space_fields([
    //         SpaceField::HostIds,
    //         SpaceField::CreatedAt,
    //         SpaceField::CreatorId,
    //         SpaceField::Id,
    //         // SpaceField::Lang,
    //         // SpaceField::InvitedUserIds,
    //         SpaceField::ParticipantCount,
    //         SpaceField::SpeakerIds,
    //         SpaceField::StartedAt,
    //         SpaceField::EndedAt,
    //         // SpaceField::SubscriberCount,
    //         SpaceField::TopicIds,
    //         SpaceField::State,
    //         SpaceField::Title,
    //         // SpaceField::UpdatedAt,
    //         SpaceField::ScheduledStart,
    //         // SpaceField::IsTicketed,
    //     ])
    //     .send()
    //     .await?
    //     .into_data()
    //     .expect("Space Not Found");
    // println!("\n\n\t\tSpace: {:?}", space);
    // let auth =
    //     BearerToken::new(std::env::var("TWITTER_APP_BEARER_TOKEN").unwrap());
    // let space = TwitterApi::new(auth)
    //     .get_space("1ypKddPokoaKW")
    //     .topic_fields([
    //         TopicField::Id,
    //         TopicField::Name,
    //         TopicField::Description,
    //     ])
    //     .user_fields([
    //         UserField::CreatedAt,
    //         UserField::Description,
    //         UserField::Id,
    //         UserField::Name,
    //         UserField::PinnedTweetId,
    //         UserField::ProfileImageUrl,
    //         UserField::PublicMetrics,
    //         UserField::Username,
    //         UserField::Verified,
    //         // Entities,
    //         // Location,
    //         // Protected,
    //         // Url,
    //         // Withheld,
    //     ])
    //     .expansions([
    //         SpaceExpansion::HostIds,
    //         SpaceExpansion::SpeakerIds,
    //         SpaceExpansion::CreatorId,
    //         // SpaceExpansion::InvitedUserIds,
    //     ])
    //     .space_fields([
    //         SpaceField::HostIds,
    //         SpaceField::CreatedAt,
    //         SpaceField::CreatorId,
    //         SpaceField::Id,
    //         SpaceField::ParticipantCount,
    //         SpaceField::SpeakerIds,
    //         SpaceField::StartedAt,
    //         SpaceField::EndedAt,
    //         SpaceField::TopicIds,
    //         SpaceField::State,
    //         SpaceField::Title,
    //         SpaceField::ScheduledStart,
    //         // SpaceField::Lang,
    //         // SpaceField::InvitedUserIds,
    //         // SpaceField::SubscriberCount,
    //         // SpaceField::UpdatedAt,
    //         // SpaceField::IsTicketed,
    //     ])
    //     .send()
    //     .await?;
    //
    // let data = space.data().expect("Expected space data");
    // println!("{:?}", data);
    //
    // println!("Looking for Tweets for Space");
    //
    // let stored_oauth2_token =
    //     std::fs::read_to_string("./.oauth2_token.json").unwrap();
    // let auth: Oauth2Token = serde_json::from_str(&stored_oauth2_token)?;
    //
    // let tweets = TwitterApi::new(auth)
    //     .get_space_tweets("1djGXlXZogOGZ")
    //     .send()
    //     .await?;
    //
    // let tweet_data = tweets.data().expect("Couldn't find tweets");
    // println!("\n\t\tTweets:{:?}", tweet_data);
    // // O-AUTH
    // // So I don't know how I should get a stored oauth2_token here
    // // So how do we make a stored Oauthtoken here
    // Ok(())
}

// async fn find_includes(
//     space: &ApiResponse<BearerToken, Space, ()>,
// ) -> Result<()> {
//     // Can we return the space tho???
//     let includes = space.into_includes().expect("expected includes");
//     Ok(())
// }

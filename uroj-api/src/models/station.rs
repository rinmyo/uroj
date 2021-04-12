use super::user::User;
use async_graphql::*;
use chrono::NaiveDateTime;
use uroj_db::models::station::Station as StationData;
use uroj_db::{get_conn_from_ctx, models::user::User as UserData};

#[derive(SimpleObject)]
#[graphql(complex)]
pub(crate) struct Station {
    id: i32,
    title: String,
    description: Option<String>,
    created: NaiveDateTime,
    updated: NaiveDateTime,
    draft: bool,
    author_id: Option<i32>,
    yaml: String,
}

#[ComplexObject]
impl Station {
    async fn author(&self, ctx: &Context<'_>) -> Option<User> {
        self.author_id.map(|uid| {
            let author = UserData::get(uid, &get_conn_from_ctx(ctx)).expect("cannot query user");
            (&author).into()
        })
    }
}

impl From<&StationData> for Station {
    fn from(station: &StationData) -> Self {
        Station {
            id: station.id,
            title: station.title.clone(),
            description: station.description.clone(),
            created: station.created,
            updated: station.updated,
            draft: station.draft,
            author_id: station.author_id,
            yaml: station.yaml.clone(),
        }
    }
}

#[derive(InputObject)]
pub(crate) struct StationInput {
    pub(crate) title: String,
    pub(crate) description: Option<String>,
    pub(crate) draft: bool,
    pub(crate) yaml: String,
}

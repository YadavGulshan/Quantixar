use axum::{
  Json,
  Router,
  routing::get,
};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub fn data_set_router() -> Router {
  let app = Router::new()
          .route("/dataset", get(list_datasets).post(create_data_set));
  app
}


#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct DataSet {
  user_id: String,
  name: String,
  description: String,
}

impl DataSet {
  pub fn new(user_id: String, name: String, description: String) -> Self {
    Self {
      user_id,
      name,
      description,
    }
  }
}

pub(crate) async fn list_datasets() -> Json<Vec<DataSet>> {
  let data_sets: Vec<DataSet> = vec![
    DataSet {
      user_id: "user_id".to_string(),
      name: "name".to_string(),
      description: "description".to_string(),
    },
  ];
  Json(data_sets)
}

pub(crate) async fn create_data_set(Json(params): Json<DataSet>) -> impl IntoResponse {
  let data_set = crate::http::hanlders::dataset::DataSet::new(params.user_id, params.name, params.description);
  Json(data_set)
}

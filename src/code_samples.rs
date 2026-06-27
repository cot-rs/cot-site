use cot_site_macros::code_sample;

pub const ORM_CODE_SAMPLE: &str = code_sample!(
    "rust",
    r#"#[model]
struct Customer {
    #[model(primary_key)]
    id: Auto<i64>,
    #[model(unique)]
    email: Email,
    is_verified: bool,
}

let customer = query!(Customer, $email == email)
    .get(&db)
    .await?;"#
);

pub const FORMS_CODE_SAMPLE: &str = code_sample!(
    "rust",
    r#"#[derive(Debug, Form)]
struct SignupForm {
    #[form(opts(max_length = 100))]
    username: String,
    email: Email,
    password: Password,
}

async fn signup(
    RequestForm(form): RequestForm<SignupForm>,
) -> cot::Result<Response> {
    let form = form.unwrap();
    // username, email, and password are already validated
    DatabaseUser::create_user(&db, &form.username, &form.password).await?;
    Ok(reverse_redirect!(urls, "index")?)
}"#
);

pub const ADMIN_CODE_SAMPLE: &str = code_sample!(
    "rust",
    r#"#[derive(Debug, Clone, Form, AdminModel)]
#[model]
struct TodoItem {
    #[model(primary_key)]
    id: Auto<i32>,
    title: String,
}

impl App for TodoApp {
    fn admin_model_managers(&self) -> Vec<Box<dyn AdminModelManager>> {
        vec![Box::new(DefaultAdminModelManager::<TodoItem>::new())]
    }
}"#
);

pub const JSON_CODE_SAMPLE: &str = code_sample!(
    "rust",
    r#"#[derive(Deserialize, schemars::JsonSchema)]
struct AddRequest { a: i32, b: i32 }

#[derive(Serialize, schemars::JsonSchema)]
struct AddResponse { result: i32 }

async fn add(Json(req): Json<AddRequest>) -> Json<AddResponse> {
    Json(AddResponse { result: req.a + req.b })
}

// Swagger UI is generated automatically from the types above
Route::with_api_handler_and_name("/add/", api_post(add), "add");"#
);

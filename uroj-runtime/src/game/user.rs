pub(crate) enum Role {
    Admin,
    User,
}
pub(crate) struct User {
    id: String,
    role: Role,
}

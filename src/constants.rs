pub const ALL_VERBS: [&str; 8] = [
    GET,
    LIST,
    WATCH,
    CREATE,
    DELETE,
    UPDATE,
    PATCH,
    DELETECOLLECTION,
];

pub const DEFAULT_VERBS: [&str; 4] = [GET, CREATE, DELETE, UPDATE];

const GET: &str = "Get";
const LIST: &str = "List";
const WATCH: &str = "Watch";
const CREATE: &str = "Create";
const DELETE: &str = "Delete";
const UPDATE: &str = "Update";
const PATCH: &str = "Patch";
const DELETECOLLECTION: &str = "DeleteCollection";

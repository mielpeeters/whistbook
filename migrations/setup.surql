// INIT login TABLE
DEFINE TABLE login SCHEMAFULL;
DEFINE FIELD email ON TABLE login TYPE string ASSERT string::is::email($value);
DEFINE FIELD pw ON TABLE login TYPE string;

DEFINE INDEX email_index ON TABLE login COLUMNS email UNIQUE;

// INIT game TABLE
DEFINE TABLE game SCHEMALESS;
DEFINE FIELD game ON TABLE game TYPE object;

// INIT plays TABLE
DEFINE TABLE plays TYPE RELATION IN login OUT game ENFORCED;
DEFINE FIELD alias ON TABLE plays TYPE string;

struct Package {
    pub name: String,
    pub local_modules: Vec<Module>,
    pub imported_packages: Vec<Package>,
}

struct Module {
    pub name: String,
    pub members: Vec<Definition>,
    pub imports: Vec<Import>
}

struct ModuleName {
    pub value: String
}

struct Import {
    pub name: ModuleName,
    pub reference: ModuleReference
}

enum ModuleReference {
    WildCard(),
    Single(),
    Many()
}

struct Definition {
    pub name: String,
    pub def_type: Expr,
    pub def_expr: Expr 
}

enum Expr {}

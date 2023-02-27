use ahash::{HashMap, HashMapExt};

use crate::typescript::*;

use super::open_api::{ApiPathOperation, OpenApi, PathArgs, ResponseOptions};

pub struct OpenApiGenerator<'m> {
    ast_map: &'m HashMap<String, AstNode>,
    declarations: HashMap<String, Declaration>,
    open_api: OpenApi,
    module_map: &'m HashMap<String, String>,
}
impl<'m> OpenApiGenerator<'m> {
    pub fn new(module_map: &'m HashMap<String, String>, ast_map: &'m HashMap<String, AstNode>) -> Self {
        OpenApiGenerator {
            ast_map,
            open_api: OpenApi::new(),
            declarations: HashMap::new(),
            module_map,
        }
    }

    pub(crate) fn result(&self) -> &super::open_api::OpenApi {
        &self.open_api
    }

    fn cache_variables(&mut self, node: &AstNode) -> () {
        if let Some(ref list) = node.declaration_list {
            list.for_each_child(|declaration| {
                let initializer = declaration.initializer.as_ref().unwrap();
                let name = declaration.name.as_ref().unwrap();
                let text = name.escaped_text.as_ref().unwrap();
                match declaration.kind {
                    AS_EXPRESSION => {
                        let _type = initializer._type.as_ref().unwrap();
                        let type_name = _type.type_name.as_ref().unwrap();
                        let type_name_text = type_name.escaped_text.as_ref().unwrap();
                        self.declarations.insert(
                            text.to_string(),
                            Declaration::Alias {
                                from: text.to_string(),
                                to: type_name_text.to_string(),
                            },
                        );
                    }
                    TYPE_ASSERTION_EXPRESSION => {
                        let _type = initializer._type.as_ref().unwrap();
                        let type_name = _type.type_name.as_ref().unwrap();
                        let type_name_text = type_name.escaped_text.as_ref().unwrap();
                        self.declarations.insert(
                            text.to_string(),
                            Declaration::Alias {
                                from: text.to_string(),
                                to: type_name_text.to_string(),
                            },
                        );
                    }
                    CALL_EXPRESSION => {
                        let expression = initializer.expression.as_ref().unwrap();
                        let expression_text = expression.escaped_text.as_ref().unwrap();
                        self.declarations.insert(
                            text.to_string(),
                            Declaration::Alias {
                                from: text.to_string(),
                                to: expression_text.to_string(),
                            },
                        );
                    }
                    NEW_EXPRESSION => {
                        let expression = initializer.expression.as_ref().unwrap();
                        let expression_text = expression.escaped_text.as_ref().unwrap();
                        self.declarations.insert(
                            text.to_string(),
                            Declaration::Alias {
                                from: text.to_string(),
                                to: expression_text.to_string(),
                            },
                        );
                    }
                    FALSE_KEYWORD => {
                        self.declarations
                            .insert(text.to_string(), Declaration::SimpleType(String::from("boolean")));
                    }
                    NUMERIC_LITERAL => {
                        self.declarations
                            .insert(text.to_string(), Declaration::SimpleType(String::from("number")));
                    }
                    STRING_LITERAL => {
                        self.declarations
                            .insert(text.to_string(), Declaration::SimpleType(String::from("string")));
                    }
                    TRUE_KEYWORD => {
                        self.declarations
                            .insert(text.to_string(), Declaration::SimpleType(String::from("boolean")));
                    }
                    _ => {
                        self.declarations
                            .insert(text.to_string(), Declaration::ComplexType { node: node.clone() });
                    }
                }
            })
        }
    }

    fn cache_object_type(&mut self, node: &AstNode) -> () {
        node.for_each_child(|child| {
            let name = child.name.as_ref().unwrap();
            let text = name.escaped_text.as_ref().unwrap();
            self.declarations
                .insert(text.to_string(), Declaration::ComplexType { node: node.clone() });
        })
    }

    fn cache_import_declaration(&mut self, node: &AstNode) -> () {
        let module_specifier = node.module_specifier.as_ref().unwrap();
        let module_reference = module_specifier.text.as_ref().unwrap();
        let absolute = self.module_map.get(module_reference).expect("Could not find module");

        let import_clause = node.import_clause.as_ref().unwrap();
        if let Some(ref name) = import_clause.name {
            let text = name.escaped_text.as_ref().unwrap();
            self.declarations.insert(
                text.to_string(),
                Declaration::DefaultImport {
                    file: absolute.to_string(),
                },
            );
        }

        import_clause.for_each_child(|bindings| {
            bindings.for_each_child(|element| {
                let name = element.name.as_ref().unwrap();
                let name_text = name.escaped_text.as_ref().unwrap();
                if let Some(ref property_name) = element.property_name {
                    let text = property_name.escaped_text.as_ref().unwrap();
                    self.declarations.insert(
                        name_text.to_string(),
                        Declaration::Import {
                            name: text.to_string(),
                            file: absolute.to_string(),
                        },
                    );
                } else {
                    self.declarations.insert(
                        name_text.to_string(),
                        Declaration::Import {
                            name: name_text.to_string(),
                            file: absolute.to_string(),
                        },
                    );
                }
            })
        })
    }

    fn cache_declarations(&mut self, node: &AstNode) -> () {
        match node.kind {
            CLASS_DECLARATION => self.cache_object_type(node),
            IMPORT_DECLARATION => self.cache_import_declaration(node),
            INTERFACE_DECLARATION => self.cache_object_type(node),
            TYPE_ALIAS_DECLARATION => self.cache_object_type(node),
            VARIABLE_STATEMENT => self.cache_variables(node),
            _ => {}
        }
    }

    fn is_api_path(&self, node: &AstNode) -> bool {
        match node.expression {
            Some(ref expression) => match expression.escaped_text {
                Some(ref text) => text.eq("Path"),
                None => false,
            },
            _ => false,
        }
    }

    fn find_api_paths(&mut self, node: &AstNode) -> () {
        if self.is_api_path(node) {
            let arguments = node.arguments.as_ref().unwrap();
            let route_handler = arguments.get(0).unwrap();
            let route_handler_body = route_handler.body.as_ref().unwrap();
            let request_param = get_request_parameter(route_handler);
            let path_options = get_path_args(arguments.get(1).unwrap());

            let route = path_options.path.unwrap();
            let mut operation = self
                .open_api
                .path(route)
                .method(path_options.method)
                .tags(path_options.tags);

            add_operation_params(request_param, operation);
            add_operation_response(&*route_handler_body, &mut operation);
        } else {
            self.cache_declarations(node);
            node.for_each_child(|node| self.find_api_paths(node))
        }
    }

    pub(crate) fn api_paths_from(&mut self, path: String) -> () {
        let source_file = self
            .ast_map
            .get(&path)
            .expect(&format!("Could not find ast for path '{}'", path));

        source_file.for_each_child(|node| self.find_api_paths(node));
    }
}

fn get_request_parameter<'n>(node: &'n AstNode) -> &'n AstNode {
    let parameters = node.parameters.as_ref().unwrap();
    parameters.first().unwrap()
}

fn get_path_args(node: &AstNode) -> PathArgs {
    let mut path_args = PathArgs::new();
    let properties = node.properties.as_ref().unwrap();
    for prop in properties {
        let name = prop.name.as_ref().unwrap().escaped_text.as_ref().unwrap();
        let initializer = prop.initializer.as_ref().unwrap();
        if name == "method" {
            path_args.method = match initializer.text {
                Some(ref text) => Some(text.clone()),
                None => None,
            };
        } else if name == "path" {
            path_args.path = match initializer.text {
                Some(ref text) => Some(text.clone()),
                None => None,
            };
        } else if name == "tags" {
            path_args.tags = match initializer.elements {
                Some(ref elements) => Some(elements.iter().map(|e| e.text.as_ref().unwrap().clone()).collect()),
                None => None,
            };
        }
    }
    path_args
}

fn add_operation_params<'cx>(node: &AstNode, operation: &mut ApiPathOperation) -> () {
    if let Some(ref _type) = node._type {
        if let Some(ref type_name) = _type.type_name {
            if let Some(ref text) = type_name.escaped_text {
                if text.eq("QueryParam") {
                    let property_name = node.name.as_ref().unwrap();
                    let name = property_name.escaped_text.as_ref().unwrap();
                    add_operation_parameter(&name, "query", operation, _type)
                }

                if text.eq("RouteParam") {
                    let property_name = node.name.as_ref().unwrap();
                    let name = property_name.escaped_text.as_ref().unwrap();
                    add_operation_parameter(&name, "path", operation, _type)
                }

                if text.eq("Header") {
                    let property_name = node.name.as_ref().unwrap();
                    let name = property_name.escaped_text.as_ref().unwrap();
                    add_operation_parameter(&name, "header", operation, _type)
                }
            }
        } else {
            _type.for_each_child(|node| add_operation_params(node, operation))
        }
    } else {
        node.for_each_child(|node| add_operation_params(node, operation))
    }
}

fn add_operation_parameter(name: &str, location: &str, operation: &mut ApiPathOperation, node: &AstNode) -> () {
    let param = operation.param(name, location);
    let arguments = node.type_arguments.as_ref().unwrap();
    if let Some(type_arg) = arguments.get(0) {
        match type_arg.kind {
            // TODO add format option to operation parameters
            NUMBER_KEYWORD => param.content().schema().primitive("number").format(None),
            STRING_KEYWORD => param.content().schema().primitive("string").format(None),
            _ => param.content().schema().reference(String::from("")),
            // TODO this probably needs the root type reference
        };
    }

    if let Some(required) = arguments.get(1) {
        let literal = required.literal.as_ref().unwrap();
        param.required(literal.kind == TRUE_KEYWORD);
    }
}

fn add_operation_response(node: &AstNode, operation: &mut ApiPathOperation) -> () {
    if let Some(ref expression) = node.expression {
        if let Some(ref text) = expression.escaped_text {
            if text.eq("Response") {
                let arguments = node.arguments.as_ref().unwrap();
                let response_type_arg = arguments.first();
                let response_options = arguments.last();
                if response_type_arg.is_some() && response_options.is_some() {
                    let response_type_arg = response_type_arg.unwrap();
                    let response_options = get_response_options(response_options.unwrap());

                    let response = operation.response(response_options);
                    let schema = response.content().schema();
                    if let Some(ref_name) = get_identifier(&response_type_arg) {
                        schema.reference(ref_name);
                    }
                    // TODO register need for references
                }
            }
        }
    } else {
        node.for_each_child(|node| add_operation_response(node, operation))
    }
}

fn get_response_options(node: &AstNode) -> ResponseOptions {
    let mut response_args = ResponseOptions::new();
    let properties = node.properties.as_ref().unwrap();
    for prop in properties {
        if let Some(prop_name) = get_identifier(prop) {
            if prop_name == "description" {
                response_args.description = prop.initializer.as_ref().unwrap().text.clone();
            } else if prop_name == "example" {
                response_args.example = prop.initializer.as_ref().unwrap().text.clone();
            } else if prop_name == "namespace" {
                response_args.namespace = prop.initializer.as_ref().unwrap().text.clone();
            } else if prop_name == "statusCode" {
                response_args.status_code = prop.initializer.as_ref().unwrap().text.clone();
            }
        }
    }

    response_args
}

fn get_identifier(cursor: &AstNode) -> Option<String> {
    if let Some(ref text) = cursor.escaped_text {
        return Some(text.clone());
    }

    if let Some(ref name) = cursor.name {
        return get_identifier(&*name);
    }

    if let Some(ref expression) = cursor.expression {
        return get_identifier(&*expression);
    }

    None
}

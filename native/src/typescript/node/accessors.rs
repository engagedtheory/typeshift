use std::{cell::RefCell, rc::Rc, vec};

use swc_ecma_ast::TsTypeElement;

use super::{Context, NodeKind, SchemyNode};

impl<'m> SchemyNode<'m> {
    pub fn args(&self) -> Vec<Rc<SchemyNode<'m>>> {
        match self.kind {
            NodeKind::CallExpr(expr) => {
                let mut borrow = self.context.borrow_mut();
                expr.args
                    .iter()
                    .map(|arg| {
                        let args_index = borrow.nodes.len();
                        borrow.nodes.push(Rc::new(SchemyNode {
                            index: args_index,
                            parent_index: Some(self.index),
                            kind: NodeKind::ExprOrSpread(arg),
                            context: self.context.clone(),
                        }));
                        borrow.nodes.get(args_index).map(|n| n.clone()).unwrap()
                    })
                    .collect()
            }
            _ => vec![],
        }
    }

    pub fn body(&self) -> Vec<Rc<SchemyNode<'m>>> {
        match self.kind {
            NodeKind::Class(class) => {
                let mut borrow = self.context.borrow_mut();
                class
                    .body
                    .iter()
                    .map(|statement| {
                        let args_index = borrow.nodes.len();
                        borrow.nodes.push(Rc::new(SchemyNode {
                            index: args_index,
                            parent_index: Some(self.index),
                            kind: NodeKind::ClassMember(statement),
                            context: self.context.clone(),
                        }));
                        borrow.nodes.get(args_index).map(|n| n.clone()).unwrap()
                    })
                    .collect()
            }
            NodeKind::TsInterfaceBody(body) => {
                let mut borrow = self.context.borrow_mut();
                body.body
                    .iter()
                    .map(|statement| {
                        let args_index = borrow.nodes.len();
                        borrow.nodes.push(Rc::new(SchemyNode {
                            index: args_index,
                            parent_index: Some(self.index),
                            kind: NodeKind::TsTypeElement(statement),
                            context: self.context.clone(),
                        }));
                        borrow.nodes.get(args_index).map(|n| n.clone()).unwrap()
                    })
                    .collect()
            }
            _ => vec![],
        }
    }

    pub fn callee(&self) -> Option<Rc<SchemyNode<'m>>> {
        match self.kind {
            NodeKind::CallExpr(expr) => {
                let mut borrow = self.context.borrow_mut();
                let callee = &expr.callee;
                let callee_index = borrow.nodes.len();
                borrow.nodes.push(Rc::new(SchemyNode {
                    index: callee_index,
                    parent_index: Some(self.index),
                    kind: NodeKind::Callee(callee),
                    context: self.context.clone(),
                }));
                borrow.nodes.get(callee_index).map(|n| n.clone())
            }
            _ => None,
        }
    }

    pub fn class(&self) -> Option<Rc<SchemyNode<'m>>> {
        match self.kind {
            NodeKind::ClassExpr(raw_expr) => {
                let mut borrow = self.context.borrow_mut();
                let class_index = borrow.nodes.len();
                borrow.nodes.push(Rc::new(SchemyNode {
                    index: class_index,
                    parent_index: Some(self.index),
                    kind: NodeKind::Class(&*raw_expr.class),
                    context: self.context.clone(),
                }));
                borrow.nodes.get(class_index).map(|n| n.clone())
            }
            NodeKind::ClassDecl(raw_decl) => {
                let mut borrow = self.context.borrow_mut();
                let class_index = borrow.nodes.len();
                borrow.nodes.push(Rc::new(SchemyNode {
                    index: class_index,
                    parent_index: Some(self.index),
                    kind: NodeKind::Class(&*raw_decl.class),
                    context: self.context.clone(),
                }));
                borrow.nodes.get(class_index).map(|n| n.clone())
            }
            _ => None,
        }
    }

    pub fn decl(&self) -> Option<Rc<SchemyNode<'m>>> {
        match self.kind {
            NodeKind::ExportDecl(export_decl) => {
                let mut borrow = self.context.borrow_mut();
                let class_index = borrow.nodes.len();
                borrow.nodes.push(Rc::new(SchemyNode {
                    index: class_index,
                    parent_index: Some(self.index),
                    kind: NodeKind::Decl(&export_decl.decl),
                    context: self.context.clone(),
                }));
                borrow.nodes.get(class_index).map(|n| n.clone())
            }
            _ => None,
        }
    }

    pub fn elem_type(&self) -> Option<Rc<SchemyNode<'m>>> {
        match self.kind {
            NodeKind::TsArrayType(array_type) => {
                let mut borrow = self.context.borrow_mut();
                let class_index = borrow.nodes.len();
                borrow.nodes.push(Rc::new(SchemyNode {
                    index: class_index,
                    parent_index: Some(self.index),
                    kind: NodeKind::TsType(&*array_type.elem_type),
                    context: self.context.clone(),
                }));
                borrow.nodes.get(class_index).map(|n| n.clone())
            }
            _ => None,
        }
    }

    pub fn get(&self, index: usize) -> Option<Rc<SchemyNode<'m>>> {
        let borrow = self.context.borrow();
        match borrow.nodes.get(index) {
            Some(node) => Some(node.clone()),
            None => None,
        }
    }

    pub fn get_context(&self) -> Rc<RefCell<Context<'m>>> {
        self.context.clone()
    }

    pub fn interface_body(&self) -> Option<Rc<SchemyNode<'m>>> {
        match self.kind {
            NodeKind::TsInterfaceDecl(decl) => {
                let mut borrow = self.context.borrow_mut();
                let callee_index = borrow.nodes.len();
                borrow.nodes.push(Rc::new(SchemyNode {
                    index: callee_index,
                    parent_index: Some(self.index),
                    kind: NodeKind::TsInterfaceBody(&decl.body),
                    context: self.context.clone(),
                }));
                borrow.nodes.get(callee_index).map(|n| n.clone())
            }
            _ => None,
        }
    }

    pub fn members(&self) -> Vec<Rc<SchemyNode<'m>>> {
        match self.kind {
            NodeKind::TsTypeLit(type_lit) => {
                let mut borrow = self.context.borrow_mut();
                type_lit
                    .members
                    .iter()
                    .map(|type_element| {
                        let params_index = borrow.nodes.len();
                        borrow.nodes.push(Rc::new(SchemyNode {
                            index: params_index,
                            parent_index: Some(self.index),
                            kind: NodeKind::TsTypeElement(type_element),
                            context: self.context.clone(),
                        }));
                        borrow.nodes.get(params_index).map(|n| n.clone()).unwrap()
                    })
                    .collect()
            }
            _ => vec![],
        }
    }

    pub fn params(&self) -> Vec<Rc<SchemyNode<'m>>> {
        match self.kind {
            NodeKind::TsTypeRef(raw_ref) => {
                if let Some(raw_params) = &raw_ref.type_params {
                    let mut borrow = self.context.borrow_mut();
                    raw_params
                        .params
                        .iter()
                        .map(|param| {
                            let params_index = borrow.nodes.len();
                            borrow.nodes.push(Rc::new(SchemyNode {
                                index: params_index,
                                parent_index: Some(self.index),
                                kind: NodeKind::TsType(&*param),
                                context: self.context.clone(),
                            }));
                            borrow.nodes.get(params_index).map(|n| n.clone()).unwrap()
                        })
                        .collect()
                } else {
                    vec![]
                }
            }
            NodeKind::ExprOrSpread(raw) => match &*raw.expr {
                swc_ecma_ast::Expr::Arrow(expr) => expr
                    .params
                    .iter()
                    .map(|param| {
                        let mut borrow = self.context.borrow_mut();
                        let params_index = borrow.nodes.len();
                        borrow.nodes.push(Rc::new(SchemyNode {
                            index: params_index,
                            parent_index: Some(self.index),
                            kind: NodeKind::Pat(param),
                            context: self.context.clone(),
                        }));
                        borrow.nodes.get(params_index).map(|n| n.clone()).unwrap()
                    })
                    .collect(),
                _ => vec![],
            },
            _ => vec![],
        }
    }

    pub fn parent(&self) -> Option<Rc<SchemyNode<'m>>> {
        let borrow = self.context.borrow();
        match self.parent_index {
            Some(index) => borrow.nodes.get(index).map(|n| n.clone()),
            None => None,
        }
    }

    pub fn specifiers(&self) -> Vec<Rc<SchemyNode<'m>>> {
        match self.kind {
            NodeKind::NamedExport(named_export) => {
                let mut borrow = self.context.borrow_mut();
                named_export
                    .specifiers
                    .iter()
                    .map(|type_element| {
                        let params_index = borrow.nodes.len();
                        borrow.nodes.push(Rc::new(SchemyNode {
                            index: params_index,
                            parent_index: Some(self.index),
                            kind: NodeKind::ExportSpecifier(type_element),
                            context: self.context.clone(),
                        }));
                        borrow.nodes.get(params_index).map(|n| n.clone()).unwrap()
                    })
                    .collect()
            }
            _ => vec![],
        }
    }

    pub fn class_prop(&self) -> Option<Rc<SchemyNode<'m>>> {
        match self.kind {
            NodeKind::ClassMember(raw_member) => match raw_member {
                swc_ecma_ast::ClassMember::ClassProp(raw_prop) => {
                    let mut borrow = self.context.borrow_mut();
                    let class_index = borrow.nodes.len();
                    borrow.nodes.push(Rc::new(SchemyNode {
                        index: class_index,
                        parent_index: Some(self.index),
                        kind: NodeKind::ClassProp(&raw_prop),
                        context: self.context.clone(),
                    }));
                    borrow.nodes.get(class_index).map(|n| n.clone())
                }
                _ => None,
            },
            _ => None,
        }
    }

    pub fn type_ann(&self) -> Option<Rc<SchemyNode<'m>>> {
        match self.kind {
            NodeKind::TsTypeAliasDecl(raw_decl) => {
                let mut borrow = self.context.borrow_mut();
                let class_index = borrow.nodes.len();
                borrow.nodes.push(Rc::new(SchemyNode {
                    index: class_index,
                    parent_index: Some(self.index),
                    kind: NodeKind::TsType(&*raw_decl.type_ann),
                    context: self.context.clone(),
                }));
                borrow.nodes.get(class_index).map(|n| n.clone())
            },
            NodeKind::TsTypeElement(TsTypeElement::TsPropertySignature(raw_prop)) => match &raw_prop.type_ann {
                Some(type_ann) => {
                    let mut borrow = self.context.borrow_mut();
                    let class_index = borrow.nodes.len();
                    borrow.nodes.push(Rc::new(SchemyNode {
                        index: class_index,
                        parent_index: Some(self.index),
                        kind: NodeKind::TsTypeAnnotation(&*type_ann),
                        context: self.context.clone(),
                    }));
                    borrow.nodes.get(class_index).map(|n| n.clone())
                }
                None => None,
            },
            NodeKind::ClassProp(class_prop) => match &class_prop.type_ann {
                Some(type_ann) => {
                    let mut borrow = self.context.borrow_mut();
                    let class_index = borrow.nodes.len();
                    borrow.nodes.push(Rc::new(SchemyNode {
                        index: class_index,
                        parent_index: Some(self.index),
                        kind: NodeKind::TsTypeAnnotation(&*type_ann),
                        context: self.context.clone(),
                    }));
                    borrow.nodes.get(class_index).map(|n| n.clone())
                }
                None => None,
            },
            _ => None,
        }
    }

    pub fn type_params(&self) -> Vec<Rc<SchemyNode<'m>>> {
        match self.kind {
            NodeKind::TsTypeRef(type_ref) => {
                if let Some(type_params) = &type_ref.type_params {
                    let mut borrow = self.context.borrow_mut();
                    type_params
                        .params
                        .iter()
                        .map(|param| {
                            let params_index = borrow.nodes.len();
                            borrow.nodes.push(Rc::new(SchemyNode {
                                index: params_index,
                                parent_index: Some(self.index),
                                kind: NodeKind::TsType(param),
                                context: self.context.clone(),
                            }));
                            borrow.nodes.get(params_index).map(|n| n.clone()).unwrap()
                        })
                        .collect()
                } else {
                    vec![]
                }
            }
            _ => vec![],
        }
    }
}

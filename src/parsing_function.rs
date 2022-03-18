use std::sync::{Arc, RwLock};

use bevy::math::Vec3;

use super::parsing::*;

#[derive(Clone, Copy, Debug)]
enum FunctionType {
    Sin,
    Cos,
    Tan,
    Abs,
}

impl FunctionType {
    fn perform_f32_func(&self, x: f32) -> f32 {
        match self {
            FunctionType::Sin => f32::sin(x),
            FunctionType::Cos => f32::cos(x),
            FunctionType::Tan => f32::tan(x),
            FunctionType::Abs => f32::abs(x),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Operator {
    Addition,
    Multiplication,
    Subtraction,
    Division,
    Exponentiation,
}

impl Operator {
    fn get_precedence(&self) -> u8 {
        match self {
            Operator::Addition => 5,
            Operator::Multiplication => 10,
            Operator::Subtraction => 5,
            Operator::Division => 10,
            Operator::Exponentiation => 15,
        }
    }

    fn run(&self, x: f32, y: f32) -> f32 {
        match *self {
            Operator::Addition => x + y,
            Operator::Multiplication => x * y,
            Operator::Subtraction => x - y,
            Operator::Division => x / y,
            Operator::Exponentiation => f32::powf(x, y),
        }
    }
}

#[derive(Clone, Debug)]
enum FuncNode {
    Time,
    PointX,
    PointY,
    PointZ,
    Int(i64),
    Float(f32),
    // BinaryOperation(OperationType, Box<FuncNode>, Box<FuncNode>),
    BuiltinFunction(FunctionType, Vec<FuncNode>),
    Parentheses(Vec<FuncNode>),
    BinaryOperationSymbol(Operator),
}

impl FuncNode {
    fn simplify_to_f32(&self, time_elapsed: f32, point_pos: Vec3) -> FuncNodeCleaned {
        match self {
            FuncNode::Time => FuncNodeCleaned::Float(time_elapsed),
            FuncNode::PointX => FuncNodeCleaned::Float(point_pos.x),
            FuncNode::PointY => FuncNodeCleaned::Float(point_pos.y),
            FuncNode::PointZ => FuncNodeCleaned::Float(point_pos.z),
            FuncNode::Int(x) => FuncNodeCleaned::Int(*x),
            FuncNode::Float(x) => FuncNodeCleaned::Float(*x),
            FuncNode::BuiltinFunction(func_type, node) => FuncNodeCleaned::BuiltinFunction(*func_type, (*node).iter().map(|x| x.simplify_to_f32(time_elapsed, point_pos)).collect()),
            FuncNode::Parentheses(nodes) => FuncNodeCleaned::Parentheses((*nodes).iter().map(|x| x.simplify_to_f32(time_elapsed, point_pos)).collect()),
            FuncNode::BinaryOperationSymbol(oper) => FuncNodeCleaned::BinaryOperationSymbol(*oper),
        }
    }
}

#[derive(Clone)]
enum FuncNodeCleaned {
    Int(i64),
    Float(f32),
    BuiltinFunction(FunctionType, Vec<FuncNodeCleaned>),
    BinaryOperationSymbol(Operator),
    Parentheses(Vec<FuncNodeCleaned>),
}

fn parse_time(input: &mut ParseInput) -> Result<FuncNode, String> {
    input.skip_word("time")?;
    Ok(FuncNode::Time)
}

fn parse_point_x(input: &mut ParseInput) -> Result<FuncNode, String> {
    input.skip_char('x')?;
    Ok(FuncNode::PointX)
}

fn parse_point_y(input: &mut ParseInput) -> Result<FuncNode, String> {
    input.skip_char('y')?;
    Ok(FuncNode::PointY)
}

fn parse_point_z(input: &mut ParseInput) -> Result<FuncNode, String> {
    input.skip_char('z')?;
    Ok(FuncNode::PointZ)
}

fn parse_binary_operation_symbol(input: &mut ParseInput) -> Result<FuncNode, String> {
    if input.skip_char('+').is_ok() {
        Ok(FuncNode::BinaryOperationSymbol(Operator::Addition))
    } else if input.skip_char('*').is_ok() {
        Ok(FuncNode::BinaryOperationSymbol(Operator::Multiplication))
    } else if input.skip_char('-').is_ok() {
        Ok(FuncNode::BinaryOperationSymbol(Operator::Subtraction))
    } else if input.skip_char('/').is_ok() {
        Ok(FuncNode::BinaryOperationSymbol(Operator::Division))
    } else if input.skip_char('^').is_ok() {
        Ok(FuncNode::BinaryOperationSymbol(Operator::Exponentiation))
    } else {
        Err(format!("Expected +, -, *, or /, but found {:?}", input.get_next_char()))
    }
}

fn parse_integer(input: &mut ParseInput) -> Result<FuncNode, String> {
    let first_char = input.pop_next_char_numerical()?;
    let mut output = first_char.to_string();
    while let Ok(next_char) = input.pop_next_char_numerical() {
        output += &next_char.to_string();
    }
    Ok(FuncNode::Int(str::parse::<i64>(&output).map_err(|err| err.to_string())?))
}

// fn parse_negation(interior_parser: Arc<dyn Fn(&mut ParseInput) -> Result<FuncNode, String>>) -> Box<dyn Fn(&mut ParseInput) -> Result<FuncNode, String>> {
//     Box::new(move | input: &mut ParseInput | {
//         input.skip_char('-')?;
//         Ok(FuncNode::NegationOperation(Box::new(interior_parser(input)?)))
//     })
// }

fn parse_builtin_func(interior_parser: Arc<dyn Fn(&mut ParseInput) -> Result<FuncNode, String> + Send + Sync>) -> Arc<dyn Fn(&mut ParseInput) -> Result<FuncNode, String> + Send + Sync> {
    Arc::new(move | input: &mut ParseInput | {
        let func_type =
            if input.match_word_ci("sin") {
                FunctionType::Sin
            } else if input.match_word_ci("cos") {
                FunctionType::Cos
            } else if input.match_word_ci("tan") {
                FunctionType::Tan
            } else if input.match_word_ci("abs") {
                FunctionType::Abs
            } else {
                return Err(format!("Expected sin, cos, or tan, but found {:?}", input.get_next_char()))
            };
        input.skip_x_chars(3);
        input.skip_char('(')?;
        let mut output: Vec<FuncNode> = Vec::new();
        loop {
            input.skip_spaces_and_newlines();
            match interior_parser(input) {
                Ok(value) => output.push(value),
                Err(e) => {
                    if input.skip_char(')').is_ok() {
                        break;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        Ok(FuncNode::BuiltinFunction(func_type, output))
    })
}

fn parse_parentheses(interior_parser: Arc<dyn Fn(&mut ParseInput) -> Result<FuncNode, String> + Send + Sync>) -> Arc<dyn Fn(&mut ParseInput) -> Result<FuncNode, String> + Send + Sync> {
    Arc::new(move | input: &mut ParseInput | {
        input.skip_char('(')?;
        let mut output: Vec<FuncNode> = Vec::new();
        loop {
            input.skip_spaces_and_newlines();
            match interior_parser(input) {
                Ok(value) => {
                    output.push(value);
                },
                Err(e) => {
                    if input.skip_char(')').is_ok() {
                        break;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        Ok(FuncNode::Parentheses(output))
    })
}

// fn parse_binary_operation(interior_parser: Arc<dyn Fn(&mut ParseInput) -> Result<FuncNode, String>>) -> Box<dyn Fn(&mut ParseInput) -> Result<FuncNode, String>> {
//     Box::new(move | input: &mut ParseInput | {
//         let in_parens = input.skip_char('(').is_ok();
//         let op_type =
//             match input.get_next_plain_char() {
//                 Some('+') => OperationType::Add,
//                 Some('-') => OperationType::Sub,
//                 Some('*') => OperationType::Mult,
//                 Some('/') => OperationType::Div,
//                 Some(c) => return Err(format!("Expected one of +, -, *, or /, but found {}", c)),
//                 None => return Err(format!("Expected one of +, -, *, or /, but found end of input")),
//             };
//         input.skip_next_char();
//         input.skip_spaces_and_newlines();
//         let first_value = interior_parser(input)?;
//         input.skip_spaces_and_newlines();
//         let second_value = interior_parser(input)?;
//         if in_parens {
//             input.skip_char(')')?;
//         }
//         Ok(FuncNode::BinaryOperation(op_type, Box::new(first_value), Box::new(second_value)))
//     })
// }

fn try_parsers_with_list<'a>(parsers: Arc<RwLock<Vec<Arc<(dyn for<'r> Fn(&'r mut ParseInput) -> Result<FuncNode, String> + Send + Sync)>>>>) -> Arc<dyn for<'r> Fn(&'r mut ParseInput) -> Result<FuncNode, String> + Sync + Send + 'a> {
    Arc::new(move | input: &mut ParseInput | -> Result<FuncNode, String> {
        let save_point = input.create_save_point();
        let mut last_err = String::new();
        match parsers.read() {
            Ok(parser_list) => {
                for parser in parser_list.iter() {
                    input.skip_spaces_and_newlines();
                    match parser(input) {
                        Ok(x) => return Ok(x),
                        Err(err) => {
                            last_err = err;
                            input.load_save_point(save_point);
                        }
                    }
                }
            },
            Err(e) => {
                return Err(e.to_string())
            },
        }
        Err(last_err.to_string())
    })
}

fn translate_nodes(nodes: &Vec<FuncNode>, time_elapsed: f32, point_pos: Vec3) -> Result<f32, String> {
    let cleaned_nodes: Vec<FuncNodeCleaned> = nodes.iter().map(|n| n.simplify_to_f32(time_elapsed, point_pos)).collect();
    compile_cleaned_nodes(cleaned_nodes)
}

fn compile_cleaned_nodes(nodes: Vec<FuncNodeCleaned>) -> Result<f32, String> {
    let mut numbers: Vec<f32> = Vec::new();
    let mut operators: Vec<Operator> = Vec::new();
    for current in nodes.iter() {
        match &*current {
            FuncNodeCleaned::BinaryOperationSymbol(oper) => {
                if let Some(stash_op) = operators.last() {
                    if stash_op.get_precedence() > oper.get_precedence() {
                        let oper = operators.pop();
                        let right_operand = numbers.pop();
                        let left_operand = numbers.pop();
                        if oper.is_none() || right_operand.is_none() || left_operand.is_none() {
                            return Err("Erroneous binary operation attempted".to_string())
                        }
                        numbers.push(oper.unwrap().run(left_operand.unwrap(), right_operand.unwrap()));
                    }
                }
                operators.push(*oper);
            },
            FuncNodeCleaned::Float(x) => numbers.push(*x),
            FuncNodeCleaned::Int(x) => numbers.push(*x as f32),
            FuncNodeCleaned::BuiltinFunction(func_type, inner_nodes) => numbers.push(func_type.perform_f32_func(compile_cleaned_nodes(inner_nodes.to_vec())?)),
            FuncNodeCleaned::Parentheses(inner_nodes) => numbers.push(compile_cleaned_nodes(inner_nodes.to_vec())?),
        }
    }

    // Cleanup remaining operations
    for operation in operators {
        let right_operand = numbers.pop();
        let left_operand = numbers.pop();
        if right_operand.is_none() || left_operand.is_none() {
            return Err("Erroneous binary operation attempted".to_string())
        }
        numbers.push(operation.run(left_operand.unwrap(), right_operand.unwrap()));
    }

    numbers.get(0).map(|x| *x).ok_or_else(|| { "Input is empty.".to_owned() })
}

// fn translate_node(node: FuncNode, time_elapsed: f32, point_pos: Vec3) -> f32 {
//     match node {
//         FuncNode::Time => time_elapsed,
//         FuncNode::PointX => point_pos.x,
//         FuncNode::PointY => point_pos.y,
//         FuncNode::PointZ => point_pos.z,
//         FuncNode::Int(x) => x as f32,
//         FuncNode::BuiltinFunction(func, value_node) => func.perform_f32_func(translate_node(*value_node, time_elapsed, point_pos)),
//         // FuncNode::BinaryOperation(op, x_node, y_node) => {
//         //     let x = translate_node(*x_node, time_elapsed, point_pos);
//         //     let y = translate_node(*y_node, time_elapsed, point_pos);
//         //     op.perform_func(x, y) 
//         // },

//     }
// }

pub struct FormulaParser {
    main_parser: Arc<dyn Fn(&mut ParseInput) -> Result<FuncNode, String> + Send + Sync>,
}

impl FormulaParser {
    pub fn new() -> Self {
        let parsers: Arc<RwLock<Vec<Arc<dyn for<'r> Fn(&'r mut ParseInput) -> Result<FuncNode, String> + Send + Sync>>>> = Arc::new(RwLock::new(vec!(
            Arc::new(parse_binary_operation_symbol),
            Arc::new(parse_time),
            Arc::new(parse_point_x),
            Arc::new(parse_point_y),
            Arc::new(parse_point_z),
            Arc::new(parse_integer),
        )));
    
        let main_parser = try_parsers_with_list(parsers.clone());
    
        // let negation_parser = parse_negation(Arc::new(main_parser));
        // parsers.borrow_mut().insert(0, Box::new(negation_parser));
    
        let mut edit_parser_list = parsers.write().unwrap();

        let builtin_func_parser = parse_builtin_func(Arc::clone(&main_parser));
        edit_parser_list.insert(0, Arc::clone(&builtin_func_parser));
    
        let parentheses_parser = parse_parentheses(Arc::clone(&main_parser));
        edit_parser_list.insert(1, Arc::clone(&parentheses_parser));

        FormulaParser {
            main_parser: Arc::clone(&main_parser),
        }
    }

    pub fn parse(&self, input: &str) -> Arc<dyn Fn(f32, Vec3) -> Result<f32, String> + Sync + Send> {
        let mut parse_input = ParseInput::new(input.to_owned());
    
        let mut output_nodes: Vec<FuncNode> = Vec::new();

        loop {
            if parse_input.finished() {
                return Arc::new(move | time_elapsed: f32, point_pos: Vec3 | -> Result<f32, String> {
                    translate_nodes(&output_nodes, time_elapsed, point_pos)
                });
            }
            match (self.main_parser)(&mut parse_input) {
                Ok(node) => {
                    output_nodes.push(node);
                },
                Err(string) => {
                    return Arc::new(move | time_elapsed: f32, point_pos: Vec3 | -> Result<f32, String> {
                        Err(string.clone())
                    });
                }
            }
        }
    }
}

// pub fn parse_sim_func(input: &str) -> Arc<dyn Fn(f32, Vec3) -> Result<f32, String> + Sync + Send> {
//     let parsers: Rc<RefCell<Vec<Box<dyn for<'r> Fn(&'r mut ParseInput) -> Result<FuncNode, String>>>>> = Rc::new(RefCell::new(vec!(
//         Box::new(parse_binary_operation_symbol),
//         Box::new(parse_time),
//         Box::new(parse_point_x),
//         Box::new(parse_point_y),
//         Box::new(parse_point_z),
//         Box::new(parse_integer),
//     )));

//     let main_parser = &*Box::leak(try_parsers_with_list(parsers.clone()));

//     // let negation_parser = parse_negation(Arc::new(main_parser));
//     // parsers.borrow_mut().insert(0, Box::new(negation_parser));

//     let builtin_func_parser = parse_builtin_func(Arc::new(main_parser));
//     parsers.borrow_mut().insert(0, Box::new(builtin_func_parser));

//     let parentheses_parser = parse_parentheses(Arc::new(main_parser));
//     parsers.borrow_mut().insert(1, Box::new(parentheses_parser));

//     // let add_parser = parse_binary_operation(Arc::new(main_parser));
//     // parsers.borrow_mut().push(add_parser);
//     // parsers.borrow_mut().insert(1, Box::new(add_parser));

//     let mut parse_input = ParseInput::new(input.to_owned());
    
//     let mut output_nodes: Vec<FuncNode> = Vec::new();

//     loop {
//         if parse_input.finished() {
//             return Arc::new(move | time_elapsed: f32, point_pos: Vec3 | -> Result<f32, String> {
//                 translate_nodes(&output_nodes, time_elapsed, point_pos)
//             });
//         }
//         match main_parser(&mut parse_input) {
//             Ok(node) => {
//                 output_nodes.push(node);
//             },
//             Err(string) => {
//                 return Arc::new(move | time_elapsed: f32, point_pos: Vec3 | -> Result<f32, String> {
//                     Err(string.clone())
//                 });
//             }
//         }
//     }
// }
use super::parser::ParseRule;

#[derive(Debug, PartialEq, Eq)]
pub enum BinaryOperator {
    Ncl,
    Mul,
    Div,
    Add,
    Sub,
    Gt,
    Lt,
    Gte,
    Lte,
    Eq,
    Neq,
    And,
    Or,
    Push,
}

impl BinaryOperator {
    pub fn from_rule(rule: &ParseRule) -> Self {
        match rule {
            ParseRule::operator_binary_ncl => BinaryOperator::Ncl,
            ParseRule::operator_binary_mul => BinaryOperator::Mul,
            ParseRule::operator_binary_div => BinaryOperator::Div,
            ParseRule::operator_binary_add => BinaryOperator::Add,
            ParseRule::operator_binary_sub => BinaryOperator::Sub,
            ParseRule::operator_binary_gt => BinaryOperator::Gt,
            ParseRule::operator_binary_lt => BinaryOperator::Lt,
            ParseRule::operator_binary_gte => BinaryOperator::Gte,
            ParseRule::operator_binary_lte => BinaryOperator::Lte,
            ParseRule::operator_binary_eq => BinaryOperator::Eq,
            ParseRule::operator_binary_neq => BinaryOperator::Neq,
            ParseRule::operator_binary_and => BinaryOperator::And,
            ParseRule::operator_binary_or => BinaryOperator::Or,
            ParseRule::operator_binary_push => BinaryOperator::Push,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum AssignmentOperator {
    Direct,
    Mul,
    Div,
    Add,
    Sub,
    And,
    Or,
    Ncl,
}

impl AssignmentOperator {
    pub fn from_rule(rule: &ParseRule) -> Self {
        match rule {
            ParseRule::operator_assign_direct => AssignmentOperator::Direct,
            ParseRule::operator_assign_ncl => AssignmentOperator::Ncl,
            ParseRule::operator_assign_mul => AssignmentOperator::Mul,
            ParseRule::operator_assign_div => AssignmentOperator::Div,
            ParseRule::operator_assign_add => AssignmentOperator::Add,
            ParseRule::operator_assign_sub => AssignmentOperator::Sub,
            ParseRule::operator_assign_and => AssignmentOperator::And,
            ParseRule::operator_assign_or => AssignmentOperator::Or,
            _ => unreachable!(),
        }
    }
}

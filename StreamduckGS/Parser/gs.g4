grammar gs;

program: scriptParameter* useImport* (eventDeclaration | functionDeclaration | constant)* EOF;

// Declaration

pluginItem: item=STRING_LITERAL AT plugin=STRING_LITERAL;
useImport: USE pluginItem AS ID;

parameter: ID;
parameters: LPARENT parameter (COMMA parameter)* RPARENT;
 
eventDeclaration: ON ID parameters? LBRACE statement* RBRACE;
functionDeclaration: FN ID parameters? LBRACE statement* RBRACE;

scriptParameterProperty: ID ASSIGNMENT literal;
scriptParameter: PARAMETER parameterVariable=ID AS parameterFieldType=STRING_LITERAL scriptParameterProperty*;

constant: CONST name=ID ASSIGNMENT (
    integer
    | FLOAT
    | STRING_LITERAL
    | (TRUE | FALSE)
);

// Statement stuff

argument: expression;
arguments: argument (COMMA argument)*;

call: anyVariable LPARENT arguments? RPARENT;
objectedCall: anyVariable COLON ID LPARENT arguments? RPARENT;

returnStatement: RETURN expression # returnSomething
                | RETURN # returnNothing;

globalVariable: GLOBAL ID # globalAccess
                | globalVariable DOT localVariable # globalNested
                | globalVariable LSQUARE expression RSQUARE # globalArrayAccess;
localVariable: ID # varLocal
        | localVariable DOT localVariable # varNested
        | localVariable LSQUARE expression RSQUARE # varArrayAccess;
anyVariable: localVariable | globalVariable;

variableAssignment: anyVariable ASSIGNMENT expression # assign
                    | anyVariable ADD_AND_ASSIGN expression # addAssign
                    | anyVariable SUB_AND_ASSIGN expression # subAssign
                    | anyVariable MUL_AND_ASSIGN expression # mulAssign
                    | anyVariable DIV_AND_ASSIGN expression # divAssign
                    | anyVariable POW_AND_ASSIGN expression # powAssign
                    | anyVariable MOD_AND_ASSIGN expression # modAssign
                    | anyVariable ASSIGN_IF_NULL expression # assignIfNull;

ifStatement: IF expression LBRACE statement* RBRACE 
            ( ELSE IF expression LBRACE statement* RBRACE)*
            ( ELSE LBRACE statement* RBRACE )?;

forLoop: FOR ID ASSIGNMENT expression TO expression (STEP expression)? LBRACE statement* RBRACE;

whileLoop: WHILE expression LBRACE statement* RBRACE;

lockStatement: LOCK globalVariable LBRACE statement* RBRACE;

codeBlock: LBRACE statement* RBRACE;

statement: ifStatement
            | forLoop
            | whileLoop
            | lockStatement
            | codeBlock
            | functionDeclaration
            | variableAssignment
            | objectedCall
            | call
            | returnStatement;

// Expression stuff

objectItem: (STRING_LITERAL | ID) COLON expression;

integer: INT | BINARY | HEX | OCTAL;

literal: integer # literalInteger
        | FLOAT # literalFloat
        | STRING_LITERAL # literalString
        | (TRUE | FALSE) # literalBoolean
        | NULL # literalNull
        | LSQUARE expression? (COMMA expression)* RSQUARE # literalArray
        | LBRACE objectItem? (COMMA objectItem)* RBRACE # literalObject;

expression: expression ADD expression # expressionMathAdd // Math
            | expression SUB expression # expressionMathSub
            | expression MUL expression # expressionMathMul
            | expression DIV expression # expressionMathDiv
            | expression POW expression # expressionMathPow
            | expression MOD expression # expressionMathMod
            
            | expression EQUAL expression # expressionEqualTo // Comparison
            | expression NOT_EQUAL expression # expressionNotEqualTo
            | expression GREATER expression # expressionGreaterThan
            | expression GREATER_EQUAL expression # expressionGreaterOrEqualThan
            | expression LESSER expression # expressionLesserThan
            | expression LESSER_EQUAL expression # expressionLesserOrEqualThan
            
            | NOT expression # expressionNot // Logic/Bitwise
            | expression OR expression # expressionOr
            | expression AND expression # expressionAnd
            | expression XOR expression # expressionXor
            | expression LSHIFT expression # expressionLeftShift
            | expression RSHIFT expression # expressionRightShift
            
            | expression NULL_COALESCE expression # expressionNullCoalesce
            
            | FN parameters? LBRACE statement* RBRACE # anonymousFunction // Functions
            | expression LPARENT arguments? RPARENT # expressionCall
            | expression COLON ID LPARENT arguments? RPARENT # expressionObjectedCall
            
            | literal # expressionLiteral // Literals
            
            | anyVariable # expressionVarValue // Variables
            
            | expression DOT localVariable # expressionObjectAccess
            | expression LSQUARE expression RSQUARE # expressionArrayAccess
            
            | IF expression THEN expression ELSE expression # ternary // Ternary
            
            | LPARENT expression RPARENT # expressionNested; // Nested expression

// Lexer down here

// Comment
COMMENT: '//' ~('\r' | '\n')* -> skip;

// Keywords
USE: 'use';
AS: 'as';
AT: 'at';
ON: 'on';
FN: 'fn';
GLOBAL: 'global';
RETURN: 'return';
IF: 'if';
THEN: 'then';
ELSE: 'else';
FOR: 'for';
TO: 'to';
STEP: 'step';
WHILE: 'while';
PARAMETER: 'parameter';
LOCK: 'lock';
CONST: 'const';

// Null related operators
ASSIGN_IF_NULL: '?=';
NULL_COALESCE: '??';

// Logic/Bitwise operators
NOT: 'not';
OR: 'or';
AND: 'and';
XOR: 'xor';
LSHIFT: 'lshift';
RSHIFT: 'rshift';

// Symbols
LPARENT: '(';
RPARENT: ')';
LBRACE: '{';
RBRACE: '}';
LSQUARE: '[';
RSQUARE: ']';
COMMA: ',';
DOT: '.';
COLON: ':';

// Comparison operators
EQUAL: '==';
NOT_EQUAL: '!=';
GREATER: '>';
LESSER: '<';
GREATER_EQUAL: '>=';
LESSER_EQUAL: '<=';

// Math operators
ADD: '+';
SUB: '-';
MUL: '*';
DIV: '/';
POW: '^';
MOD: '%';

// Assignment operators
ASSIGNMENT: '=';
ADD_AND_ASSIGN: '+=';
SUB_AND_ASSIGN: '-=';
MUL_AND_ASSIGN: '*=';
DIV_AND_ASSIGN: '/=';
POW_AND_ASSIGN: '^=';
MOD_AND_ASSIGN: '%=';

// Literals
STRING_LITERAL : '"' (~('"' | '\\' ) | '\\' ('"' | '\\'))* '"';
FLOAT: '-'?[0-9_]+('.' [0-9_]+)?;
BINARY: '0b' [01_]+;
HEX: '0x' [0-9a-fA-F_]+;
OCTAL: '0o' [0-7_]+;
INT: '-'?[0-9_]+;
TRUE: 'true';
FALSE: 'false';
NULL: 'null';

ID: [a-zA-Z_][a-zA-Z_\-0-9]*;

WS: [ \t\n]+ -> skip;
ANY: .;
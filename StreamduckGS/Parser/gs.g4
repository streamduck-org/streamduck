grammar gs;

program: useImport* (eventDeclaration | functionDeclaration)* EOF;

// Declaration

pluginItem: item=STRING_LITERAL AT plugin=STRING_LITERAL;
useImport: USE pluginItem AS ID;

parameter: ID;
parameters: LPAREN parameter (COMMA parameter)* RPAREN;
 
eventBody: (statement)*;
eventDeclaration: ON ID parameters? LBRACE eventBody RBRACE;

functionBody: (statement | returnStatement)*;
functionDeclaration: FN ID parameters? LBRACE functionBody RBRACE;



// Statement stuff

argument: expression;
arguments: argument (COMMA argument)*;

call: ID LPAREN arguments? RPAREN;
returnStatement: RETURN expression;

variable: ID;
globalVariable: GLOBAL ID;

variableAssignment: (variable | globalVariable) ASSIGNMENT expression;

ifStatement: IF expression LBRACE statement* RBRACE 
            ( ELSE IF expression LBRACE statement* RBRACE)*
            ( ELSE LBRACE statement* RBRACE )?;

forLoop: FOR ID ASSIGNMENT expression TO expression (STEP expression)? LBRACE statement* RBRACE;

whileLoop: WHILE expression LBRACE statement* RBRACE;

statement: ifStatement
            | forLoop
            | whileLoop
            | variableAssignment
            | call;

// Expression stuff

expression: expression ADD expression # mathAdd // Math
            | expression SUB expression # mathSub
            | expression MUL expression # mathMul
            | expression DIV expression # mathDiv
            | expression POW expression # mathPow
            
            | expression EQUAL expression # equalTo // Comparison
            | expression NOT_EQUAL expression # notEqualTo
            | expression GREATER expression # greaterThan
            | expression GREATER_EQUAL expression # greaterOrEqualThan
            | expression LESSER expression # lesserThan
            | expression LESSER_EQUAL expression # lesserOrEqualThan
            
            | NOT expression # notOperation // Logic
            | expression OR expression # orOperation
            | expression AND expression # andOperation
            | expression XOR expression # xorOperation
            
            | call # codeCall // Action/Function calls
            
            | INT # intLiteral // Literals
            | FLOAT # floatLiteral
            | STRING_LITERAL # stringLiteral
            | (TRUE | FALSE) # boolean
            
            | (variable | globalVariable) # variableValue // Variables
            
            | LPAREN expression RPAREN # nested; // Nested expression

// Lexer down here

// Keywords
USE: 'use';
AS: 'as';
AT: 'at';
ON: 'on';
FN: 'fn';
GLOBAL: 'global';
RETURN: 'return';
IF: 'if';
ELSE: 'else';
FOR: 'for';
TO: 'to';
STEP: 'step';
WHILE: 'while';

// Logic operators
NOT: 'not';
OR: 'or';
AND: 'and';
XOR: 'xor';

// Organization symbols
LPAREN: '(';
RPAREN: ')';
LBRACE: '{';
RBRACE: '}';
COMMA: ',';

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

ASSIGNMENT: '=';

// Literals
STRING_LITERAL : '"' (~('"' | '\\' ) | '\\' ('"' | '\\'))* '"';
FLOAT: '-'?[0-9]+('.' [0-9]+)?;
INT: '-'?[0-9]+;
TRUE: 'true';
FALSE: 'false';

ID: [a-zA-Z][a-zA-Z_\-0-9]*;

WS: [ \t\n]+ -> skip;
ANY: .;
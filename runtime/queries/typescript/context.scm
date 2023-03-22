(arrow_function
	body: (_) @context.end
) @context

(function_declaration
	body: (_) @context.end
) @context

(
  (expression_statement
  	(call_expression
      	function: (_)))
  (statement_block) @context.end
) @context

[
	(arrow_function)
	(call_expression)
	(class_declaration)
  (else_clause)
  (expression_statement)
  (for_statement)
  (function_declaration)
  (interface_declaration)
  (interface_declaration)
  (lexical_declaration)
  (lexical_declaration)
  (method_definition)
  (object)
  (pair)
  (while_statement)
] @context


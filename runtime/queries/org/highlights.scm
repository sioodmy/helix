(headline
  (stars) @markup.heading.marker
  (item) @markup.heading)

(item . (expr) @keyword.directive (#match? @keyword.directive "^(TODO|DONE|NEXT|WAITING)$"))

(tag_list (tag) @label)

(timestamp) @constant.datetime

(directive name: (expr) @keyword.directive (value)? @string)

(property_drawer) @comment
(property name: (expr) @variable.parameter (value)? @string)

(comment) @comment

(drawer name: (expr) @keyword.directive)
(block name: (expr) @keyword.directive)
(dynamic_block name: (expr) @keyword.directive)

(bullet) @markup.list.unnumbered
(checkbox) @punctuation.special

((expr) @markup.bold (#match? @markup.bold "^\\*.*\\*$"))
((expr) @markup.italic (#match? @markup.italic "^/.*/$"))
((expr) @markup.raw.inline (#match? @markup.raw.inline "^~.*~$"))
((expr) @markup.quote (#match? @markup.quote "^=.*=$"))
((expr) @markup.strikethrough (#match? @markup.strikethrough "^\\+.*\\+$"))
((expr) @markup.link.url (#match? @markup.link.url "^\\[\\[.*\\]\\]$"))
((expr) @markup.math (#match? @markup.math "^\\$\\$.*\\$\\$$"))

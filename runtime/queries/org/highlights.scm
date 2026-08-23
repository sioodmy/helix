(headline
  (stars) @markup.heading.marker
  (item) @markup.heading)

(item . (expr) @keyword.directive (#match? @keyword.directive "^(TODO|DONE|NEXT|WAITING)$"))

(tag_list (tag) @label)

(timestamp) @comment
(plan) @comment

(directive) @comment

(property_drawer) @comment
(property) @comment

(comment) @comment

(drawer) @comment
(block) @comment
(dynamic_block) @comment

(bullet) @markup.list.unnumbered
(checkbox) @punctuation.special

((expr) @markup.bold (#match? @markup.bold "^\\*.*\\*$"))
((expr) @markup.italic (#match? @markup.italic "^/.*/$"))
((expr) @markup.raw.inline (#match? @markup.raw.inline "^~.*~$"))
((expr) @markup.quote (#match? @markup.quote "^=.*=$"))
((expr) @markup.strikethrough (#match? @markup.strikethrough "^\\+.*\\+$"))
((expr) @comment (#match? @comment "^\\[\\[.*\\]\\]$"))
((expr) @comment (#match? @comment "^\\$\\$.*\\$\\$$"))
((expr) @comment (#match? @comment "^(SCHEDULED:|DEADLINE:|CLOSED:)$"))

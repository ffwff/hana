#pragma once

struct vm;
struct hana_ast {};
struct hana_ast *hana_parse(const char *str);
struct hana_ast *hana_parse_file(const char *str);
void hana_ast_emit(struct hana_ast *, struct vm*);
void hana_free_ast(struct hana_ast *);

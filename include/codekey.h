/* CodeKey C API — Vietnamese Telex/VNI engine */
#ifndef CODEKEY_H
#define CODEKEY_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>

typedef struct CodeKeyEngine CodeKeyEngine;

/* method: 0 = Telex, 1 = VNI */
CodeKeyEngine *codekey_engine_new(int method);
void codekey_engine_free(CodeKeyEngine *eng);
void codekey_string_free(char *s);

/*
 * Feed Unicode code point. Return codes:
 * 0 Update, 1 Append, 2 CommitAndPass, 3 Commit, 4 Backspace, 5 Ignored
 */
int codekey_engine_feed(CodeKeyEngine *eng, uint32_t ch);
int codekey_engine_backspace(CodeKeyEngine *eng);

char *codekey_engine_preedit(const CodeKeyEngine *eng);
char *codekey_engine_commit(CodeKeyEngine *eng);
void codekey_engine_reset(CodeKeyEngine *eng);
void codekey_engine_set_enabled(CodeKeyEngine *eng, int enabled);
int codekey_engine_is_enabled(const CodeKeyEngine *eng);

char *codekey_transform(int method, const char *input);

#ifdef __cplusplus
}
#endif

#endif /* CODEKEY_H */

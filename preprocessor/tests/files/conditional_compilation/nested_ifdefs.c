#define TMP 0

#ifdef TMP
#define OTHER 1
#ifdef OTHER
int first;
#endif
#else
int second;
#endif

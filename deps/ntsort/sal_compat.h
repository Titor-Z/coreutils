// GCC/MinGW compatibility shim for MSVC SAL annotations
// This file is included automatically via -include in build.rs

#ifndef SAL_COMPAT_H
#define SAL_COMPAT_H

#define __nullterminated
#define __inout
#define __in
#define __in_ecount(x)
#define __out_ecount(x)

// GetTempPath2 is a Windows 10 1903+ API not available in older MinGW headers
#ifndef GetTempPath2
#define GetTempPath2(cch, path) GetTempPathW(cch, path)
#endif

#endif

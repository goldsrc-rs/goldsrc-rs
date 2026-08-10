// MSVC linker directives to export Rust functions with clean names
#pragma comment(linker, "/EXPORT:GiveFnptrsToDll=_GiveFnptrsToDll@8")
#pragma comment(linker, "/EXPORT:Meta_Query=_Meta_Query")
#pragma comment(linker, "/EXPORT:Meta_Attach=_Meta_Attach")
#pragma comment(linker, "/EXPORT:Meta_Detach=_Meta_Detach")

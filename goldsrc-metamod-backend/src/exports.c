// MSVC linker directives to export Rust functions
#pragma comment(linker, "/EXPORT:GiveFnptrsToDll=_GiveFnptrsToDll@8")
#pragma comment(linker, "/EXPORT:Meta_Query=_Meta_Query")
#pragma comment(linker, "/EXPORT:Meta_Attach=_Meta_Attach")
#pragma comment(linker, "/EXPORT:Meta_Detach=_Meta_Detach")
#pragma comment(linker, "/EXPORT:GetEntityAPI2=_GetEntityAPI2")
#pragma comment(linker, "/EXPORT:GetEntityAPI2_Post=_GetEntityAPI2_Post")
#pragma comment(linker, "/EXPORT:GetNewDLLFunctions=_GetNewDLLFunctions")

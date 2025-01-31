package cpp

//#cgo LDFLAGS: ${SRCDIR}/../vendor/cpp/parser.o
//#cgo LDFLAGS: ${SRCDIR}/../vendor/cpp/scanner.o
//#cgo LDFLAGS: -lstdc++
//#cgo CFLAGS: -I${SRCDIR}/../vendor/tree-sitter/lib/include
//#include "tree_sitter/parser.h"
//TSLanguage *tree_sitter_cpp();
import "C"
import (
	"unsafe"

	sitter "github.com/smacker/go-tree-sitter"
)

func GetLanguage() *sitter.Language {
	ptr := unsafe.Pointer(C.tree_sitter_cpp())
	return sitter.NewLanguage(ptr)
}

module example.com/servico

go 1.22

require github.com/gin-gonic/gin v1.10.0

require (
	github.com/google/uuid v1.6.0
	golang.org/x/text v0.14.0 // indirect
)

exclude github.com/bad/pkg v1.0.0

replace example.com/old => ./local/old

toolchain go1.22.3

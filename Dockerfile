FROM golang:1.19 AS build
WORKDIR /go/src/cidar
COPY . .
RUN go mod download
RUN CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -tags release -a -installsuffix cgo -o app .
FROM alpine:latest
WORKDIR /app
RUN mkdir ./static
COPY ./ ./
COPY --from=build /go/src/cidar/app .
CMD ["./app"]
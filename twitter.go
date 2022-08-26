package main

import (
	"context"
	"encoding/json"
	"fmt"
	"github.com/dghubble/oauth1"
	"github.com/g8rswimmer/go-twitter/v2"
	"log"
	"net/http"
	"os"
	"runtime"
)

var TwitterClient *twitter.Client

type authorize struct {
}

func (a authorize) Add(req *http.Request) {
}

func init() {
	ApiKey = os.Getenv("API_KEY")
	ApiSecret = os.Getenv("API_SECRET")
	BearerToken = os.Getenv("BEARER_TOKEN")
	AccessToken = os.Getenv("ACCESS_TOKEN")
	AccessSecret = os.Getenv("ACCESS_SECRET")

	config := oauth1.NewConfig(ApiKey, ApiSecret)
	httpClient := config.Client(oauth1.NoContext, &oauth1.Token{
		Token:       AccessToken,
		TokenSecret: ApiSecret,
	})

	TwitterClient := &twitter.Client{
		Authorizer: &authorize{},
		Client:     httpClient,
		Host:       "https://api.twitter.com",
	}

	req := twitter.CreateTweetRequest{
		Text: "This is a test tweet from the Discord Cidar bot: " + runtime.Version(),
	}
	fmt.Println("Callout to create tweet callout")

	tweetResponse, err := TwitterClient.CreateTweet(context.Background(), req)
	if err != nil {
		log.Panicf("create tweet error: %v", err)
	}

	enc, err := json.MarshalIndent(tweetResponse, "", "    ")
	if err != nil {
		log.Panic(err)
	}
	fmt.Println(string(enc))
}

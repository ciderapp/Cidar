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
	AccessToken = os.Getenv("ACCESS_TOKEN")
	AccessSecret = os.Getenv("ACCESS_SECRET")

	if *debug {
		log.Println(ApiKey)
		log.Println(ApiSecret)
		log.Println(AccessToken)
		log.Println(AccessSecret)
	}

	config := oauth1.NewConfig(AccessToken, AccessSecret)
	httpClient := config.Client(oauth1.NoContext, &oauth1.Token{
		Token:       ApiKey,
		TokenSecret: ApiSecret,
	})

	TwitterClient = &twitter.Client{
		Authorizer: &authorize{},
		Client:     httpClient,
		Host:       "https://api.twitter.com",
	}

	req := twitter.CreateTweetRequest{
		Text: "This is a test tweet from the Discord Cidar bot: " + runtime.Version(),
	}

	tweetResponse, err := TwitterClient.CreateTweet(context.Background(), req)
	if err != nil {
		log.Printf("create tweet error: %v", err)
		return
	}

	enc, err := json.MarshalIndent(tweetResponse, "", "    ")
	if err != nil {
		log.Panic(err)
	}
	fmt.Println(string(enc))
	//in := &gotwi.NewClientInput{
	//	AuthenticationMethod: gotwi.AuthenMethodOAuth1UserContext,
	//	OAuthToken:           AccessToken,
	//	OAuthTokenSecret:     AccessSecret,
	//}
	//
	//c, err := gotwi.NewClient(in)
	//if err != nil {
	//	fmt.Println(err)
	//	return
	//}
	//
	//p := &types.CreateInput{
	//	Text: gotwi.String("This is a test tweet from the Discord Cidar bot: " + runtime.Version()),
	//}
	//
	//_, err = managetweet.Create(context.Background(), c, p)
	//if err != nil {
	//	fmt.Println(err.Error())
	//	return
	//}
}

package main

import (
	"context"
	"github.com/michimani/gotwi"
	"github.com/michimani/gotwi/tweet/managetweet"
	"github.com/michimani/gotwi/tweet/managetweet/types"
	"log"
)

var TwitterClient *gotwi.Client

func init() {
	var err error
	ClientInput := &gotwi.NewClientInput{
		AuthenticationMethod: gotwi.AuthenMethodOAuth1UserContext,
		OAuthToken:           ApiKey,
		OAuthTokenSecret:     ApiSecret,
	}

	TwitterClient, err = gotwi.NewClient(ClientInput)
	if err != nil {
		log.Println(err)
		return
	}

	Input := &types.CreateInput{
		Text: gotwi.String("Test tweet from the Cidar discord bot"),
	}

	_, err = managetweet.Create(context.Background(), TwitterClient, Input)
	if err != nil {
		log.Println(err.Error())
		return
	}
}

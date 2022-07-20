package main

import (
	"errors"
	"io"
	"io/ioutil"
	"log"
	"net/http"
)

const AppleApiUrl = "https://api.music.apple.com/"

func RequestEndpoint(method string, endpoint string, body io.Reader) ([]byte, error) {
	client := http.Client{}
	req, err := http.NewRequest(method, AppleApiUrl+endpoint, body)
	if err != nil {
		log.Println(err)
	}

	req.Header = http.Header{
		"Authorization": []string{"Bearer " + DeveloperToken},
	}

	res, err := client.Do(req)
	if res.Body == nil {
		log.Println(err)
	}

	bodyBytes, err := ioutil.ReadAll(res.Body)
	if err != nil {
		log.Println(err)
	}
	if res.StatusCode != 200 {
		return nil, errors.New("non 200: " + res.Status)
	}
	return bodyBytes, nil
}

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
		"Authorization":  []string{"Bearer " + DeveloperToken},
		"DNT":            []string{"1"},
		"authority":      []string{"amp-api.music.apple.com"},
		"origin":         []string{"https://beta.music.apple.com"},
		"referer":        []string{"https://beta.music.apple.com"},
		"sec-fetch-dest": []string{"empty"},
		"sec-fetch-mode": []string{"cors"},
		"sec-fetch-site": []string{"same-site"},
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

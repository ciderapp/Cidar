package main

import (
	"encoding/json"
	"fmt"
	"github.com/bwmarrin/discordgo"
	"github.com/joho/godotenv"
	"github.com/mileusna/crontab"
	"io/ioutil"
	"log"
	"net/http"
	"net/url"
	"os"
	"os/signal"
	"regexp"
	"strings"
	"syscall"
)

var discordSession *discordgo.Session

var DeveloperToken string

var urlRegexp *regexp.Regexp
var appleRegexp *regexp.Regexp
var spotifyRegexp *regexp.Regexp

func main() {
	log.SetPrefix("[Cidar] ")

	log.Println("Starting discord bot")
	_, hasToken := os.LookupEnv("TOKEN")
	_, hasWebhookID := os.LookupEnv("WEBHOOK_ID")
	if !hasToken || !hasWebhookID {
		err := godotenv.Load()
		if err != nil {
			log.Println("Could not load dotenv file")
			return
		}
	}
	discordSession, err := discordgo.New(fmt.Sprintf("Bot %s", os.Getenv("TOKEN")))
	if err != nil {
		log.Println(fmt.Sprintf("err:%s", err.Error()))
	}

	discordSession.AddHandler(func(s *discordgo.Session, r *discordgo.Ready) {
		log.Printf("Logged in as: %v#%v", s.State.User.Username, s.State.User.Discriminator)
	})
	err = discordSession.Open()
	if err != nil {
		log.Fatalf("Cannot open the session: %v", err)
	}
	discordSession.Identify.Intents = discordgo.IntentGuildMessages

	// Setup regex early so we dont compile each message
	urlRegexp, err = regexp.Compile("(?:(?:https?|ftp)://)?[\\w/\\-?=%.]+\\.[\\w/\\-&?=%.]+")
	if err != nil {
		log.Fatalln("Failed to compile url regex")
	}

	appleRegexp, err = regexp.Compile("music.apple.com/(.+[a-z](/?)+)")
	if err != nil {
		log.Fatalln("Failed to compile apple regex")
	}

	spotifyRegexp, err = regexp.Compile("open.spotify.com/(.+[a-z](/?)+)")
	if err != nil {
		log.Fatalln("Failed to compile spotify regex")
	}

	// Cronjobs
	ctab := crontab.New()
	ctab.MustAddJob("0 * * * *", func() { // Every Hour
		var client http.Client
		req, err := http.NewRequest("GET", "https://api.cider.sh/v1", nil)
		if err != nil {
			log.Println(err)
		}

		req.Header = http.Header{
			"User-Agent": []string{"Cider"},
		}

		res, err := client.Do(req)
		if res.Body == nil {
			log.Println(err)
		}

		body, err := ioutil.ReadAll(res.Body)
		if err != nil {
			log.Println(err)
		}

		var tokenJson map[string]interface{}
		err = json.Unmarshal(body, &tokenJson)
		if err != nil {
			log.Println(err)
		}
		DeveloperToken = tokenJson["token"].(string)
	}) // Every 30 minutes

	ctab.RunAll()

	discordSession.AddHandler(test)
	defer discordSession.Close()
	sc := make(chan os.Signal, 1)
	signal.Notify(sc, syscall.SIGINT, syscall.SIGTERM, os.Interrupt, os.Kill)
	<-sc
}

func test(session *discordgo.Session, message *discordgo.MessageCreate) {
	if message.Author.ID == session.State.User.ID {
		return
	}

	if urlRegexp.MatchString(message.Content) {
		messageUrl := urlRegexp.FindString(message.Content)
		origMessageUrl := messageUrl
		if !spotifyRegexp.MatchString(messageUrl) && !appleRegexp.MatchString(messageUrl) {
			return
		}
		if spotifyRegexp.MatchString(messageUrl) {
			req, err := http.Get("https://api.song.link/v1-alpha.1/links?url=" + messageUrl)
			if err != nil {
				log.Println(err)
				return
			}
			if req.StatusCode != 200 {
				log.Println("non 200", req.Status)
				return
			}
			b, err := ioutil.ReadAll(req.Body)
			if err != nil {
				log.Println(err)
				return
			}
			var songs Link
			err = json.Unmarshal(b, &songs)
			if err != nil {
				log.Println(err)
				return
			}
			messageUrl = songs.LinksByPlatform.AppleMusic.URL
		}
		uri, _ := url.ParseRequestURI(messageUrl)
		values := uri.Query()
		id := values.Get("i")
		client := http.Client{}
		req, err := http.NewRequest("GET", fmt.Sprintf("https://api.music.apple.com/v1/catalog/%s/songs/%s", "us", id), nil)
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

		body, err := ioutil.ReadAll(res.Body)
		if err != nil {
			log.Println(err)
		}
		if res.StatusCode != 200 {
			return
		}

		var song Song
		err = json.Unmarshal(body, &song)
		if err != nil {
			log.Println(err)
		}

		song.Data[0].Attributes.Artwork.URL = strings.ReplaceAll(song.Data[0].Attributes.Artwork.URL, "{w}x{h}", "512x512")

		modLink := strings.ReplaceAll(song.Data[0].Attributes.URL, "https://", "")
		playLink := "https://cider.sh/p?" + modLink
		viewLink := "https://cider.sh/o?" + modLink

		if err != nil {
			log.Println(err)
		}
		content := ""
		if len(strings.TrimSpace(strings.ReplaceAll(message.Content, origMessageUrl, ""))) != 0 {
			content = strings.ReplaceAll(message.Content, origMessageUrl, "(embed)")
		}

		webhook, err := session.Webhook(os.Getenv("WEBHOOK_ID"))
		if err != nil {
			log.Println(err)
			return
		}

		useWebhook, hasUseWebhook := os.LookupEnv("USE_WEBHOOK")
		if hasUseWebhook && useWebhook == "true" {
			_, err = session.WebhookExecute(webhook.ID, webhook.Token, false, &discordgo.WebhookParams{
				AvatarURL: message.Author.AvatarURL(""),
				Username:  message.Author.Username,
				Content:   content,
				Embeds: []*discordgo.MessageEmbed{{
					Title:       song.Data[0].Attributes.Name,
					Color:       16449599,
					URL:         song.Data[0].Attributes.URL,
					Thumbnail:   &discordgo.MessageEmbedThumbnail{URL: song.Data[0].Attributes.Artwork.URL},
					Description: song.Data[0].Attributes.AlbumName + "\n" + song.Data[0].Attributes.ArtistName,
					// Footer:      &discordgo.MessageEmbedFooter{Text: "Shared by " + message.Author.Username, IconURL: message.Author.AvatarURL("")},
				}},
				Components: []discordgo.MessageComponent{
					discordgo.ActionsRow{
						Components: []discordgo.MessageComponent{
							discordgo.Button{
								Label: "Play In Cider",
								Style: discordgo.LinkButton,
								URL:   playLink,
							},
							discordgo.Button{
								Label: "View In Cider",
								Style: discordgo.LinkButton,
								URL:   viewLink,
							},
						},
					},
				},
			})
			if err != nil {
				log.Println(err)
				return
			}
		} else {
			seconds := song.Data[0].Attributes.DurationInMillis / 1000
			ss := seconds % 60
			mm := (seconds / 60) % 60
			hh := (seconds / (60 * 60)) % 24
			t := fmt.Sprintf("%d:%02d:%02d", hh, mm, ss)

			_, err = session.ChannelMessageSendComplex(
				message.ChannelID,
				&discordgo.MessageSend{Embed: &discordgo.MessageEmbed{
					Title:       song.Data[0].Attributes.Name,
					Color:       16449599,
					URL:         song.Data[0].Attributes.URL,
					Thumbnail:   &discordgo.MessageEmbedThumbnail{URL: song.Data[0].Attributes.Artwork.URL},
					Description: "Listen to " + song.Data[0].Attributes.AlbumName + " by " + song.Data[0].Attributes.ArtistName + " on Cider\n" + "Album: " + song.Data[0].Attributes.AlbumName + "\nRelease: " + song.Data[0].Attributes.ReleaseDate + "\nDuration: " + t,
					Footer:      &discordgo.MessageEmbedFooter{Text: "Shared by " + message.Author.Username, IconURL: message.Author.AvatarURL("")},
				}, Components: []discordgo.MessageComponent{
					discordgo.ActionsRow{
						Components: []discordgo.MessageComponent{
							discordgo.Button{
								Label: "Play In Cider",
								Style: discordgo.LinkButton,
								URL:   playLink,
							},
							discordgo.Button{
								Label: "View In Cider",
								Style: discordgo.LinkButton,
								URL:   viewLink,
							},
						},
					},
				}},
			)
			if err != nil {
				log.Println(err)
				return
			}
		}
		_ = session.ChannelMessageDelete(message.ChannelID, message.ID)
	}
}

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
	"path"
	"regexp"
	"strconv"
	"strings"
	"syscall"
)

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
	if message.Author.ID == session.State.User.ID || len(message.WebhookID) > 0 {
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
			defer req.Body.Close()
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
			if len(messageUrl) < 1 {
				_, _ = session.ChannelMessageSendReply(message.ChannelID, "Could not convert Spotify link to Apple Music link", message.Reference())
				return
			}
		}
		uri, _ := url.ParseRequestURI(messageUrl)
		values := uri.Query()
		id := values.Get("i")
		var body []byte
		var err error
		var t string

		var title string
		var urlEmbed string
		var thumbnail string
		var description string
		var footer string

		if len(id) == 0 {
			if strings.Contains(uri.Path, "album") {
				body, err = RequestEndpoint("GET", fmt.Sprintf("v1/catalog/%s/albums/%s", "us", path.Base(uri.Path)), nil)
				if err != nil {
					log.Println(err)
					return
				}
				var album Album
				err = json.Unmarshal(body, &album)
				if err != nil {
					log.Println(err)
				}
				var totalMills int
				for i := 0; i < len(album.Data[0].Relationships.Tracks.Data); i++ {
					totalMills += album.Data[0].Relationships.Tracks.Data[i].Attributes.DurationInMillis
				}
				if totalMills > 0 {
					t = MilliscondsToHHMMSS(totalMills)
				}
				title = album.Data[0].Attributes.Name
				urlEmbed = album.Data[0].Attributes.URL
				thumbnail = strings.ReplaceAll(album.Data[0].Attributes.Artwork.URL, "{w}x{h}", "512x512")
				description = "Listen to " + album.Data[0].Attributes.Name + " by " + album.Data[0].Attributes.ArtistName + " on Cider"
				footer = "Shared by " + message.Author.Username + "#" + message.Author.Discriminator + " | Songs: " + strconv.Itoa(len(album.Data[0].Relationships.Tracks.Data)) + " • Duration: " + t
			} else if strings.Contains(uri.Path, "playlist") {
				body, err = RequestEndpoint("GET", fmt.Sprintf("v1/catalog/%s/playlists/%s", "us", path.Base(uri.Path)), nil)
				if err != nil {
					log.Println(err)
					return
				}
				var playlist Playlist
				err = json.Unmarshal(body, &playlist)
				if err != nil {
					log.Println(err)
				}
				var totalMills int
				for i := 0; i < len(playlist.Data[0].Relationships.Tracks.Data); i++ {
					totalMills += playlist.Data[0].Relationships.Tracks.Data[i].Attributes.DurationInMillis
				}
				if totalMills > 0 {
					t = MilliscondsToHHMMSS(totalMills)
				}
				title = playlist.Data[0].Attributes.Name
				urlEmbed = playlist.Data[0].Attributes.URL
				thumbnail = strings.ReplaceAll(playlist.Data[0].Attributes.Artwork.URL, "{w}x{h}", "512x512")
				description = "Listen to " + playlist.Data[0].Attributes.Name + " by " + playlist.Data[0].Attributes.CuratorName + " on Cider"
				footer = "Shared by " + message.Author.Username + "#" + message.Author.Discriminator + " | Songs: " + strconv.Itoa(len(playlist.Data[0].Relationships.Tracks.Data)) + " • Duration: " + t
			}
		} else if strings.Contains(uri.Path, "song") {
			body, err = RequestEndpoint("GET", fmt.Sprintf("v1/catalog/%s/songs/%s", "us", id), nil)
			if err != nil {
				log.Println(err)
				return
			}
			var song Song
			err = json.Unmarshal(body, &song)
			if err != nil {
				log.Println(err)
			}
			if song.Data[0].Attributes.DurationInMillis > 0 {
				t = MilliscondsToHHMMSS(song.Data[0].Attributes.DurationInMillis)
			}
			title = song.Data[0].Attributes.Name
			urlEmbed = song.Data[0].Attributes.URL
			thumbnail = strings.ReplaceAll(song.Data[0].Attributes.Artwork.URL, "{w}x{h}", "512x512")
			description = "Listen to " + song.Data[0].Attributes.AlbumName + " by " + song.Data[0].Attributes.ArtistName + " on Cider"
			footer = "Shared by " + message.Author.Username + "#" + message.Author.Discriminator + " | " + t + " • " + song.Data[0].Attributes.ReleaseDate
		} else {
			_, _ = session.ChannelMessageSendReply(message.ChannelID, "Apple music link type is not implemented", message.Reference())
			return
		}

		modLink := strings.ReplaceAll(urlEmbed, "https://", "")
		if len(modLink) == 0 {
			return
		}
		playLink := "https://cider.sh/p?" + modLink
		viewLink := "https://cider.sh/o?" + modLink

		if err != nil {
			log.Println(err)
		}
		content := ""
		if len(strings.TrimSpace(strings.ReplaceAll(message.Content, origMessageUrl, ""))) != 0 {
			content = strings.ReplaceAll(message.Content, origMessageUrl, "(embed)")
		}

		useWebhook, hasUseWebhook := os.LookupEnv("USE_WEBHOOK")
		if hasUseWebhook && useWebhook == "true" {
			webhook, err := session.WebhookCreate(message.ChannelID, "temporary-cidar", "")
			if err != nil {
				log.Println(err)
				return
			}
			_, err = session.WebhookExecute(webhook.ID, webhook.Token, false, &discordgo.WebhookParams{
				AvatarURL: message.Author.AvatarURL(""),
				Username:  message.Author.Username,
				Content:   content,
				Embeds: []*discordgo.MessageEmbed{{
					Title:       title,
					Color:       16449599,
					URL:         urlEmbed,
					Thumbnail:   &discordgo.MessageEmbedThumbnail{URL: thumbnail},
					Description: description,
					Footer:      &discordgo.MessageEmbedFooter{Text: footer, IconURL: message.Author.AvatarURL("")},
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
			err = session.WebhookDelete(webhook.ID)
			if err != nil {
				log.Println(err)
			}
		} else {
			_, err = session.ChannelMessageSendComplex(
				message.ChannelID,
				&discordgo.MessageSend{Embed: &discordgo.MessageEmbed{
					Title:       title,
					Color:       16449599,
					URL:         urlEmbed,
					Thumbnail:   &discordgo.MessageEmbedThumbnail{URL: thumbnail},
					Description: description,
					Footer:      &discordgo.MessageEmbedFooter{Text: footer, IconURL: message.Author.AvatarURL("")},
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

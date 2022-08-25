package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"github.com/bwmarrin/discordgo"
	"github.com/joho/godotenv"
	"github.com/mileusna/crontab"
	"io"
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
var ApiKey string
var ApiSecret string
var BearerToken string

var urlRegexp *regexp.Regexp
var appleRegexp *regexp.Regexp
var spotifyRegexp *regexp.Regexp

var debug *bool

func init() {
	log.SetPrefix("[Cidar] ")
	debug = flag.Bool("debug", false, "Enables debugging")
	flag.Parse()
	if debug != nil && *debug {
		log.SetFlags(log.Flags() | log.Lshortfile)
	}
}

func main() {
	log.Println("Starting discord bot")

	err := godotenv.Load()
	if *debug {
		log.Println("Loading dotenv")
	}
	if err != nil {
		log.Println("Dotenv not found")
	}

	token := os.Getenv("TOKEN")
	ApiKey = os.Getenv("API_KEY")
	ApiSecret = os.Getenv("API_SECRET")
	BearerToken = os.Getenv("BEARER_TOKEN")
	if *debug {
		log.Println("Token: ", token)
	}

	discordSession, err := discordgo.New(fmt.Sprintf("Bot %s", token))
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

	// Setup regex early so we don't compile each message
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

		body, err := io.ReadAll(res.Body)
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

	discordSession.AddHandler(CiderEmbed)
	discordSession.AddHandler(MonochromishCucker)
	defer func(discordSession *discordgo.Session) {
		err := discordSession.Close()
		if err != nil {
			log.Println(err)
		}
	}(discordSession)
	sc := make(chan os.Signal, 1)
	signal.Notify(sc, syscall.SIGINT, syscall.SIGTERM, os.Interrupt, os.Kill)
	<-sc
}

func MonochromishCucker(session *discordgo.Session, message *discordgo.MessageCreate) {
	if message.Author.ID == session.State.User.ID || len(message.WebhookID) > 0 {
		return
	}
	if message.Author.ID != "500315184510795819" {
		return
	}

	content := strings.ToLower(message.Content)

	if strings.Contains(content, "rest") && strings.Contains(content, "api") {
		err := session.ChannelMessageDelete(message.ChannelID, message.ID)
		if err != nil {
			log.Println(err)
			return
		}
		resp, err := http.Get("https://cdn.discordapp.com/attachments/995118775852605501/1001300325795385496/yStHlfeya2hijvTO.mp4")
		if resp.StatusCode != 200 {
			return
		}

		_, err = session.ChannelMessageSendComplex(
			message.ChannelID,
			&discordgo.MessageSend{
				File: &discordgo.File{Reader: resp.Body, Name: "FuckOff.mp4"},
			},
		)
		if err != nil {
			log.Println(err)
			return
		}
	}
}

func CiderEmbed(session *discordgo.Session, message *discordgo.MessageCreate) {
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
			defer func(Body io.ReadCloser) {
				err := Body.Close()
				if err != nil {
					log.Println(err)
				}
			}(req.Body)
			b, err := io.ReadAll(req.Body)
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

		var body []byte
		var err error
		var t string

		var title string
		var urlEmbed string
		var thumbnail string
		var description string
		var footer string

		storefront := strings.ReplaceAll(uri.Path, "https://", "")
		subPaths := strings.Split(storefront, "/")
		storefront = subPaths[1]
		if strings.Contains(uri.Path, "song") || values.Has("i") {
			id := values.Get("i")
			body, err = RequestEndpoint("GET", fmt.Sprintf("v1/catalog/%s/songs/%s", storefront, id), nil)
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
				t = MillisecondsToHHMMSS(song.Data[0].Attributes.DurationInMillis)
			}
			title = song.Data[0].Attributes.Name
			urlEmbed = song.Data[0].Attributes.URL
			thumbnail = ThumbnailLink(song.Data[0].Attributes.Artwork.URL, 512, 512)
			description = "Listen to " + song.Data[0].Attributes.AlbumName + " by " + song.Data[0].Attributes.ArtistName + " on Cider"
			footer = "Shared by " + message.Author.Username + "#" + message.Author.Discriminator + " | " + t + " • " + song.Data[0].Attributes.ReleaseDate
		} else if strings.Contains(uri.Path, "album") {
			body, err = RequestEndpoint("GET", fmt.Sprintf("v1/catalog/%s/albums/%s", storefront, path.Base(uri.Path)), nil)
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
				t = MillisecondsToHHMMSS(totalMills)
			}
			title = album.Data[0].Attributes.Name
			urlEmbed = album.Data[0].Attributes.URL
			thumbnail = ThumbnailLink(album.Data[0].Attributes.Artwork.URL, 512, 512)
			description = "Listen to " + album.Data[0].Attributes.Name + " by " + album.Data[0].Attributes.ArtistName + " on Cider"
			footer = "Shared by " + message.Author.Username + "#" + message.Author.Discriminator + " | Songs: " + strconv.Itoa(len(album.Data[0].Relationships.Tracks.Data)) + " • Duration: " + t
		} else if strings.Contains(uri.Path, "playlist") {
			body, err = RequestEndpoint("GET", fmt.Sprintf("v1/catalog/%s/playlists/%s", storefront, path.Base(uri.Path)), nil)
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
				t = MillisecondsToHHMMSS(totalMills)
			}
			title = playlist.Data[0].Attributes.Name
			urlEmbed = playlist.Data[0].Attributes.URL
			thumbnail = ThumbnailLink(playlist.Data[0].Attributes.Artwork.URL, 512, 512)
			description = "Listen to " + playlist.Data[0].Attributes.Name + " by " + playlist.Data[0].Attributes.CuratorName + " on Cider"
			footer = "Shared by " + message.Author.Username + "#" + message.Author.Discriminator + " | Songs: " + strconv.Itoa(len(playlist.Data[0].Relationships.Tracks.Data)) + " • Duration: " + t
		} else if strings.Contains(uri.Path, "music-video") {
			body, err = RequestEndpoint("GET", fmt.Sprintf("v1/catalog/%s/music-videos/%s", storefront, path.Base(uri.Path)), nil)
			if err != nil {
				log.Println(err)
				return
			}
			var video Video
			err = json.Unmarshal(body, &video)
			if err != nil {
				log.Println(err)
			}
			if video.Data[0].Attributes.DurationInMillis > 0 {
				t = MillisecondsToHHMMSS(video.Data[0].Attributes.DurationInMillis)
			}
			title = video.Data[0].Attributes.Name
			urlEmbed = video.Data[0].Attributes.URL
			thumbnail = ThumbnailLink(video.Data[0].Attributes.Artwork.URL, 512, 512)
			description = "Listen to " + video.Data[0].Attributes.Name + " by " + video.Data[0].Attributes.ArtistName + " on Cider"
			footer = "Shared by " + message.Author.Username + "#" + message.Author.Discriminator + " | " + t + " • " + video.Data[0].Attributes.ReleaseDate
		} else if strings.Contains(uri.Path, "artist") {
			body, err = RequestEndpoint("GET", fmt.Sprintf("v1/catalog/%s/artists/%s", storefront, path.Base(uri.Path)), nil)
			if err != nil {
				log.Println(err)
				return
			}
			var artist Artist
			err = json.Unmarshal(body, &artist)
			if err != nil {
				log.Println(err)
			}
			title = artist.Data[0].Attributes.Name
			urlEmbed = artist.Data[0].Attributes.URL
			thumbnail = ThumbnailLink(artist.Data[0].Attributes.Artwork.URL, 512, 512)
			description = "Listen to " + artist.Data[0].Attributes.Name + " on Cider"
			footer = "Shared by " + message.Author.Username + "#" + message.Author.Discriminator
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

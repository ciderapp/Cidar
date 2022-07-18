package main

import (
	"fmt"
	"github.com/bwmarrin/discordgo"
	"github.com/joho/godotenv"
	"log"
	"os"
	"os/signal"
	"regexp"
	"syscall"
)

var discordSession *discordgo.Session
var urlRegexp *regexp.Regexp
var appleRegexp *regexp.Regexp
var spotifyRegexp *regexp.Regexp

func main() {
	log.SetPrefix("[Cidar] ")

	log.Println("Starting discord bot")
	err := godotenv.Load()
	if err != nil {
		log.Println("Could not load dotenv file")
		return
	}
	discordSession, err = discordgo.New(fmt.Sprintf("Bot %s", os.Getenv("TOKEN")))
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

	discordSession.AddHandler(test)
	defer discordSession.Close()
	sc := make(chan os.Signal, 1)
	signal.Notify(sc, syscall.SIGINT, syscall.SIGTERM, os.Interrupt, os.Kill)
	<-sc
}

func test(session *discordgo.Session, message *discordgo.MessageCreate) {
	if message.GuildID != "922716538212081664" {
		return
	}
	if message.Author.ID == session.State.User.ID {
		return
	}
	log.Println(message.Content)
	if urlRegexp.MatchString(message.Content) {
		url := urlRegexp.FindString(message.Content)
		log.Println(url)
		if appleRegexp.MatchString(url) {
			log.Println("apple")
		} else if spotifyRegexp.MatchString(url) {
			log.Println("spotify")
		} else {
			log.Println("nether")
		}
	}
	if message.Content == "maringaming" {
		session.ChannelMessageSend(message.ChannelID, "yees")
	}
}

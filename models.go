package main

import "time"

type Song struct {
	Data []struct {
		ID         string `json:"id"`
		Type       string `json:"type"`
		Href       string `json:"href"`
		Attributes struct {
			Previews []struct {
				URL string `json:"url"`
			} `json:"previews"`
			Artwork struct {
				Width      int    `json:"width"`
				Height     int    `json:"height"`
				URL        string `json:"url"`
				BgColor    string `json:"bgColor"`
				TextColor1 string `json:"textColor1"`
				TextColor2 string `json:"textColor2"`
				TextColor3 string `json:"textColor3"`
				TextColor4 string `json:"textColor4"`
			} `json:"artwork"`
			ArtistName           string   `json:"artistName"`
			URL                  string   `json:"url"`
			DiscNumber           int      `json:"discNumber"`
			GenreNames           []string `json:"genreNames"`
			DurationInMillis     int      `json:"durationInMillis"`
			ReleaseDate          string   `json:"releaseDate"`
			IsAppleDigitalMaster bool     `json:"isAppleDigitalMaster"`
			Name                 string   `json:"name"`
			Isrc                 string   `json:"isrc"`
			HasLyrics            bool     `json:"hasLyrics"`
			AlbumName            string   `json:"albumName"`
			PlayParams           struct {
				ID   string `json:"id"`
				Kind string `json:"kind"`
			} `json:"playParams"`
			TrackNumber  int    `json:"trackNumber"`
			ComposerName string `json:"composerName"`
		} `json:"attributes"`
		Relationships struct {
			Artists struct {
				Href string `json:"href"`
				Data []struct {
					ID   string `json:"id"`
					Type string `json:"type"`
					Href string `json:"href"`
				} `json:"data"`
			} `json:"artists"`
			Albums struct {
				Href string `json:"href"`
				Data []struct {
					ID   string `json:"id"`
					Type string `json:"type"`
					Href string `json:"href"`
				} `json:"data"`
			} `json:"albums"`
		} `json:"relationships"`
	} `json:"data"`
}

type Album struct {
	Data []struct {
		ID         string `json:"id"`
		Type       string `json:"type"`
		Href       string `json:"href"`
		Attributes struct {
			Artwork struct {
				Width      int    `json:"width"`
				Height     int    `json:"height"`
				URL        string `json:"url"`
				BgColor    string `json:"bgColor"`
				TextColor1 string `json:"textColor1"`
				TextColor2 string `json:"textColor2"`
				TextColor3 string `json:"textColor3"`
				TextColor4 string `json:"textColor4"`
			} `json:"artwork"`
			ArtistName          string   `json:"artistName"`
			IsSingle            bool     `json:"isSingle"`
			URL                 string   `json:"url"`
			IsComplete          bool     `json:"isComplete"`
			GenreNames          []string `json:"genreNames"`
			TrackCount          int      `json:"trackCount"`
			IsMasteredForItunes bool     `json:"isMasteredForItunes"`
			ReleaseDate         string   `json:"releaseDate"`
			Name                string   `json:"name"`
			RecordLabel         string   `json:"recordLabel"`
			Upc                 string   `json:"upc"`
			Copyright           string   `json:"copyright"`
			PlayParams          struct {
				ID   string `json:"id"`
				Kind string `json:"kind"`
			} `json:"playParams"`
			IsCompilation bool `json:"isCompilation"`
		} `json:"attributes"`
		Relationships struct {
			Artists struct {
				Href string `json:"href"`
				Data []struct {
					ID   string `json:"id"`
					Type string `json:"type"`
					Href string `json:"href"`
				} `json:"data"`
			} `json:"artists"`
			Tracks struct {
				Href string `json:"href"`
				Data []struct {
					ID         string `json:"id"`
					Type       string `json:"type"`
					Href       string `json:"href"`
					Attributes struct {
						Previews []struct {
							URL string `json:"url"`
						} `json:"previews"`
						Artwork struct {
							Width      int    `json:"width"`
							Height     int    `json:"height"`
							URL        string `json:"url"`
							BgColor    string `json:"bgColor"`
							TextColor1 string `json:"textColor1"`
							TextColor2 string `json:"textColor2"`
							TextColor3 string `json:"textColor3"`
							TextColor4 string `json:"textColor4"`
						} `json:"artwork"`
						ArtistName           string   `json:"artistName"`
						URL                  string   `json:"url"`
						DiscNumber           int      `json:"discNumber"`
						GenreNames           []string `json:"genreNames"`
						DurationInMillis     int      `json:"durationInMillis"`
						ReleaseDate          string   `json:"releaseDate"`
						IsAppleDigitalMaster bool     `json:"isAppleDigitalMaster"`
						Name                 string   `json:"name"`
						Isrc                 string   `json:"isrc"`
						HasLyrics            bool     `json:"hasLyrics"`
						AlbumName            string   `json:"albumName"`
						PlayParams           struct {
							ID   string `json:"id"`
							Kind string `json:"kind"`
						} `json:"playParams"`
						TrackNumber int `json:"trackNumber"`
					} `json:"attributes"`
				} `json:"data"`
			} `json:"tracks"`
		} `json:"relationships"`
	} `json:"data"`
}

type Playlist struct {
	Data []struct {
		ID         string `json:"id"`
		Type       string `json:"type"`
		Href       string `json:"href"`
		Attributes struct {
			Artwork struct {
				Width  int    `json:"width"`
				Height int    `json:"height"`
				URL    string `json:"url"`
			} `json:"artwork"`
			IsChart          bool      `json:"isChart"`
			URL              string    `json:"url"`
			LastModifiedDate time.Time `json:"lastModifiedDate"`
			Name             string    `json:"name"`
			PlaylistType     string    `json:"playlistType"`
			CuratorName      string    `json:"curatorName"`
			PlayParams       struct {
				ID          string `json:"id"`
				Kind        string `json:"kind"`
				VersionHash string `json:"versionHash"`
			} `json:"playParams"`
			Description struct {
				Standard string `json:"standard"`
			} `json:"description"`
		} `json:"attributes"`
		Relationships struct {
			Tracks struct {
				Href string `json:"href"`
				Data []struct {
					ID         string `json:"id"`
					Type       string `json:"type"`
					Href       string `json:"href"`
					Attributes struct {
						Previews []struct {
							URL string `json:"url"`
						} `json:"previews"`
						Artwork struct {
							Width      int    `json:"width"`
							Height     int    `json:"height"`
							URL        string `json:"url"`
							BgColor    string `json:"bgColor"`
							TextColor1 string `json:"textColor1"`
							TextColor2 string `json:"textColor2"`
							TextColor3 string `json:"textColor3"`
							TextColor4 string `json:"textColor4"`
						} `json:"artwork"`
						ArtistName           string   `json:"artistName"`
						URL                  string   `json:"url"`
						DiscNumber           int      `json:"discNumber"`
						GenreNames           []string `json:"genreNames"`
						DurationInMillis     int      `json:"durationInMillis"`
						ReleaseDate          string   `json:"releaseDate"`
						IsAppleDigitalMaster bool     `json:"isAppleDigitalMaster"`
						Name                 string   `json:"name"`
						Isrc                 string   `json:"isrc"`
						HasLyrics            bool     `json:"hasLyrics"`
						AlbumName            string   `json:"albumName"`
						PlayParams           struct {
							ID   string `json:"id"`
							Kind string `json:"kind"`
						} `json:"playParams"`
						TrackNumber int `json:"trackNumber"`
					} `json:"attributes,omitempty"`
				} `json:"data"`
			} `json:"tracks"`
			Curator struct {
				Href string `json:"href"`
				Data []struct {
					ID   string `json:"id"`
					Type string `json:"type"`
				} `json:"data"`
			} `json:"curator"`
		} `json:"relationships"`
	} `json:"data"`
}

type Link struct {
	EntityUniqueID     string `json:"entityUniqueId"`
	UserCountry        string `json:"userCountry"`
	PageURL            string `json:"pageUrl"`
	EntitiesByUniqueID struct {
		ANGHAMISONG64932046 struct {
			ID              string   `json:"id"`
			Type            string   `json:"type"`
			Title           string   `json:"title"`
			ArtistName      string   `json:"artistName"`
			ThumbnailURL    string   `json:"thumbnailUrl"`
			ThumbnailWidth  int      `json:"thumbnailWidth"`
			ThumbnailHeight int      `json:"thumbnailHeight"`
			APIProvider     string   `json:"apiProvider"`
			Platforms       []string `json:"platforms"`
		} `json:"ANGHAMI_SONG::64932046"`
		DEEZERSONG65084466 struct {
			ID              string   `json:"id"`
			Type            string   `json:"type"`
			Title           string   `json:"title"`
			ArtistName      string   `json:"artistName"`
			ThumbnailURL    string   `json:"thumbnailUrl"`
			ThumbnailWidth  int      `json:"thumbnailWidth"`
			ThumbnailHeight int      `json:"thumbnailHeight"`
			APIProvider     string   `json:"apiProvider"`
			Platforms       []string `json:"platforms"`
		} `json:"DEEZER_SONG::65084466"`
		ITUNESSONG606982746 struct {
			ID              string   `json:"id"`
			Type            string   `json:"type"`
			Title           string   `json:"title"`
			ArtistName      string   `json:"artistName"`
			ThumbnailURL    string   `json:"thumbnailUrl"`
			ThumbnailWidth  int      `json:"thumbnailWidth"`
			ThumbnailHeight int      `json:"thumbnailHeight"`
			APIProvider     string   `json:"apiProvider"`
			Platforms       []string `json:"platforms"`
		} `json:"ITUNES_SONG::606982746"`
		NAPSTERSONGTra77238782 struct {
			ID              string   `json:"id"`
			Type            string   `json:"type"`
			Title           string   `json:"title"`
			ArtistName      string   `json:"artistName"`
			ThumbnailURL    string   `json:"thumbnailUrl"`
			ThumbnailWidth  int      `json:"thumbnailWidth"`
			ThumbnailHeight int      `json:"thumbnailHeight"`
			APIProvider     string   `json:"apiProvider"`
			Platforms       []string `json:"platforms"`
		} `json:"NAPSTER_SONG::tra.77238782"`
		SOUNDCLOUDSONG129691312 struct {
			ID          string   `json:"id"`
			Type        string   `json:"type"`
			Title       string   `json:"title"`
			ArtistName  string   `json:"artistName"`
			APIProvider string   `json:"apiProvider"`
			Platforms   []string `json:"platforms"`
		} `json:"SOUNDCLOUD_SONG::129691312"`
		SPOTIFYSONG68A9AyH6EO3LbZOPZEMMrl struct {
			ID              string   `json:"id"`
			Type            string   `json:"type"`
			Title           string   `json:"title"`
			ArtistName      string   `json:"artistName"`
			ThumbnailURL    string   `json:"thumbnailUrl"`
			ThumbnailWidth  int      `json:"thumbnailWidth"`
			ThumbnailHeight int      `json:"thumbnailHeight"`
			APIProvider     string   `json:"apiProvider"`
			Platforms       []string `json:"platforms"`
		} `json:"SPOTIFY_SONG::68A9AyH6eO3LbZOPZEMMrl"`
		YOUTUBEVIDEOOFEfWbhKAko struct {
			ID              string   `json:"id"`
			Type            string   `json:"type"`
			Title           string   `json:"title"`
			ArtistName      string   `json:"artistName"`
			ThumbnailURL    string   `json:"thumbnailUrl"`
			ThumbnailWidth  int      `json:"thumbnailWidth"`
			ThumbnailHeight int      `json:"thumbnailHeight"`
			APIProvider     string   `json:"apiProvider"`
			Platforms       []string `json:"platforms"`
		} `json:"YOUTUBE_VIDEO::oFEfWbhKAko"`
	} `json:"entitiesByUniqueId"`
	LinksByPlatform struct {
		Anghami struct {
			Country        string `json:"country"`
			URL            string `json:"url"`
			EntityUniqueID string `json:"entityUniqueId"`
		} `json:"anghami"`
		Deezer struct {
			Country        string `json:"country"`
			URL            string `json:"url"`
			EntityUniqueID string `json:"entityUniqueId"`
		} `json:"deezer"`
		Napster struct {
			Country        string `json:"country"`
			URL            string `json:"url"`
			EntityUniqueID string `json:"entityUniqueId"`
		} `json:"napster"`
		Soundcloud struct {
			Country        string `json:"country"`
			URL            string `json:"url"`
			EntityUniqueID string `json:"entityUniqueId"`
		} `json:"soundcloud"`
		Spotify struct {
			Country             string `json:"country"`
			URL                 string `json:"url"`
			NativeAppURIDesktop string `json:"nativeAppUriDesktop"`
			EntityUniqueID      string `json:"entityUniqueId"`
		} `json:"spotify"`
		Youtube struct {
			Country        string `json:"country"`
			URL            string `json:"url"`
			EntityUniqueID string `json:"entityUniqueId"`
		} `json:"youtube"`
		YoutubeMusic struct {
			Country        string `json:"country"`
			URL            string `json:"url"`
			EntityUniqueID string `json:"entityUniqueId"`
		} `json:"youtubeMusic"`
		AppleMusic struct {
			Country             string `json:"country"`
			URL                 string `json:"url"`
			NativeAppURIMobile  string `json:"nativeAppUriMobile"`
			NativeAppURIDesktop string `json:"nativeAppUriDesktop"`
			EntityUniqueID      string `json:"entityUniqueId"`
		} `json:"appleMusic"`
		Itunes struct {
			Country             string `json:"country"`
			URL                 string `json:"url"`
			NativeAppURIMobile  string `json:"nativeAppUriMobile"`
			NativeAppURIDesktop string `json:"nativeAppUriDesktop"`
			EntityUniqueID      string `json:"entityUniqueId"`
		} `json:"itunes"`
	} `json:"linksByPlatform"`
}

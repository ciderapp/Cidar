package main

import (
	"fmt"
	"strconv"
	"strings"
)

func MilliscondsToHHMMSS(milliseconds int) string {
	seconds := milliseconds / 1000
	ss := seconds % 60
	mm := (seconds / 60) % 60
	hh := (seconds / (60 * 60)) % 24
	if hh == 0 && mm != 0 {
		return fmt.Sprintf("%02d:%02d", mm, ss)
	} else if hh == 0 && mm == 0 {
		return fmt.Sprintf("%02d", ss)
	} else {
		return fmt.Sprintf("%d:%02d:%02d", hh, mm, ss)
	}
}

func ThumbnailLink(url string, width int, height int) string {
	url = strings.ReplaceAll(url, "{w}", strconv.Itoa(width))
	url = strings.ReplaceAll(url, "{h}", strconv.Itoa(height))
	return url
}

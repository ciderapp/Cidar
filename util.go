package main

import "fmt"

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

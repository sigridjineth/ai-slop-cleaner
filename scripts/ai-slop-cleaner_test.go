package main

import (
	"bufio"
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"math"
	"os"
	"regexp"
	"strings"
	"testing"
	"time"
)

func isolateSlopIgnore(t *testing.T) {
	t.Helper()
	t.Setenv("HOME", t.TempDir())
}

func repeatedWords(word string, count int) string {
	words := make([]string, count)
	for i := range words {
		words[i] = word
	}
	return strings.Join(words, " ")
}

func TestRunScoreFlagPrintsSingleInteger(t *testing.T) {
	isolateSlopIgnore(t)
	input := strings.Join([]string{
		"# Comprehensive Landscape",
		"**핵심은** this section will delve into the transformative landscape.",
		"This chapter showcases robust patterns. This chapter leverages seamless workflows. This chapter unlocks pivotal insights.",
		"AI is not a tool, it is a game-changer.",
		"- First, orchestrate the journey.",
		"- Second, navigate the hurdles.",
		"- Third, elucidate the tapestry.",
		"정리하면 이 장에서는 핵심은 반복 구조입니다.",
	}, "\n")

	var stdout bytes.Buffer
	var stderr bytes.Buffer
	code := run([]string{"-score"}, strings.NewReader(input), &stdout, &stderr)
	if code != 0 {
		t.Fatalf("run returned %d, stderr=%q", code, stderr.String())
	}
	if !regexp.MustCompile(`^\d+\n$`).MatchString(stdout.String()) {
		t.Fatalf("score output should be one integer line, got %q", stdout.String())
	}
	if stderr.Len() != 0 {
		t.Fatalf("score mode should not emit stderr, got %q", stderr.String())
	}
}

func TestScoreFlagReadsInputFile(t *testing.T) {
	isolateSlopIgnore(t)
	inputFile, err := os.CreateTemp(t.TempDir(), "draft-*.md")
	if err != nil {
		t.Fatal(err)
	}
	if _, err := inputFile.WriteString("Let's dive in. 핵심은 plain scoring. This is not simple, it is nuanced.\n"); err != nil {
		t.Fatal(err)
	}
	if err := inputFile.Close(); err != nil {
		t.Fatal(err)
	}

	var stdout bytes.Buffer
	code := run([]string{"-score", "-input", inputFile.Name()}, strings.NewReader("ignored"), &stdout, &bytes.Buffer{})
	if code != 0 {
		t.Fatalf("run returned %d", code)
	}
	if !regexp.MustCompile(`^\d+\n$`).MatchString(stdout.String()) {
		t.Fatalf("score output should be one integer line, got %q", stdout.String())
	}
}

func TestOverallFormulaRoundsOnce(t *testing.T) {
	got := scoreFromComponents(0.50, 0.30, 0.40, 0.50, 0.25)
	if got != 39 {
		t.Fatalf("scoreFromComponents() = %d, want worked-example score 39", got)
	}
}

func TestBannedWordDensityUsesCanonicalWeightsAndPhraseExclusion(t *testing.T) {
	isolateSlopIgnore(t)
	text := repeatedWords("plain", 990) + " delve into leverage"
	breakdown := calculateSlopScore(text)

	if breakdown.WeightedBannedHits != 3 {
		t.Fatalf("weighted banned hits = %d, want 3 (phrase*2 + single word)", breakdown.WeightedBannedHits)
	}
	want := (1000 * 3.0 / float64(breakdown.WordCount)) / 16.0
	if math.Abs(breakdown.BannedWordDensity-want) > 0.0001 {
		t.Fatalf("BWD = %.4f, want %.4f", breakdown.BannedWordDensity, want)
	}
}

func TestScoreIgnoresCodeBlocksForBannedRhythmAndMeta(t *testing.T) {
	isolateSlopIgnore(t)
	text := strings.Join([]string{
		"```",
		"Let's dive in and delve into the comprehensive landscape.",
		"This chapter highlights pivotal choices. This chapter highlights pivotal choices.",
		"```",
		"Plain prose stays outside code.",
	}, "\n")
	breakdown := calculateSlopScore(text)

	if breakdown.WeightedBannedHits != 0 {
		t.Fatalf("code-block banned hits = %d, want 0", breakdown.WeightedBannedHits)
	}
	if breakdown.MetaHits != 0 {
		t.Fatalf("code-block meta hits = %d, want 0", breakdown.MetaHits)
	}
	if breakdown.RhythmMonotony != 0 || breakdown.RhythmSampleConfidence != "low" {
		t.Fatalf("rhythm = %.4f/%s, want 0/low", breakdown.RhythmMonotony, breakdown.RhythmSampleConfidence)
	}
}

func TestMetaAndMarkdownPromptDensities(t *testing.T) {
	isolateSlopIgnore(t)
	text := strings.Join([]string{
		repeatedWords("plain", 996),
		"Let's dive in.",
		"# Heading",
		"- bullet",
		"**bold**",
	}, "\n")
	breakdown := calculateSlopScore(text)

	metaWant := (1000 * 1.0 / float64(breakdown.WordCount)) / 8.0
	if math.Abs(breakdown.MetaCommentaryDensity-metaWant) > 0.0001 {
		t.Fatalf("META = %.4f, want %.4f", breakdown.MetaCommentaryDensity, metaWant)
	}
	if breakdown.MarkdownHits != 3 {
		t.Fatalf("markdown hits = %d, want 3", breakdown.MarkdownHits)
	}
	markdownWant := (1000 * 3.0 / float64(breakdown.WordCount)) / 12.0
	if math.Abs(breakdown.MarkdownOveruse-markdownWant) > 0.0001 {
		t.Fatalf("MD = %.4f, want %.4f", breakdown.MarkdownOveruse, markdownWant)
	}
}

func TestRhythmRequiresFiveSentences(t *testing.T) {
	isolateSlopIgnore(t)
	breakdown := calculateSlopScore("One clear sentence. Another clear sentence. A third clear sentence. Fourth sentence here.")
	if breakdown.RhythmMonotony != 0 || breakdown.RhythmSampleConfidence != "low" {
		t.Fatalf("rhythm = %.4f/%s, want 0/low", breakdown.RhythmMonotony, breakdown.RhythmSampleConfidence)
	}
}

func TestSlopScoreRanges(t *testing.T) {
	isolateSlopIgnore(t)
	clean := strings.Join([]string{
		"The server reads the file, parses each line, and prints a result.",
		"Short checks catch bad input early.",
		"A clear message helps the user fix the problem.",
	}, " ")
	if got := scoreSlop(clean); got > 20 {
		t.Fatalf("clean prose score = %d, want <= 20", got)
	}

	sloppy := strings.Join([]string{
		"## Robust Landscape",
		"**In this section** we delve into a comprehensive and transformative landscape.",
		"This chapter highlights pivotal choices. This chapter highlights seamless workflows. This chapter highlights nuanced outcomes.",
		"The result is not a helper, it is a game-changer.",
		"- First, orchestrate the journey.",
		"- Second, navigate the hurdles.",
		"- Third, elucidate the tapestry.",
		"- Fourth, unlock a beacon.",
		"- Fifth, leverage synergy.",
		"Let's unpack this. 정리하면 핵심은 이 장에서는 메타 설명을 반복한다는 점입니다.",
	}, "\n")
	if got := scoreSlop(sloppy); got < 70 {
		t.Fatalf("sloppy prose score = %d, want >= 70", got)
	}
}

func TestDefaultReportModeStillWorks(t *testing.T) {
	isolateSlopIgnore(t)
	var stdout bytes.Buffer
	var stderr bytes.Buffer
	code := run([]string{}, strings.NewReader("Let's dive in and delve into the landscape.\n"), &stdout, &stderr)
	if code != 0 {
		t.Fatalf("run returned %d, stderr=%q", code, stderr.String())
	}
	if !strings.Contains(stdout.String(), "# AI Slop Cleaner Report") {
		t.Fatalf("default output should remain a report, got %q", stdout.String())
	}
}

func TestMCPInitializeHandshake(t *testing.T) {
	isolateSlopIgnore(t)
	stdinReader, stdinWriter := io.Pipe()
	stdoutReader, stdoutWriter := io.Pipe()
	var stderr bytes.Buffer

	done := make(chan int, 1)
	go func() {
		defer stdoutWriter.Close()
		done <- serveMCP(stdinReader, stdoutWriter, &stderr)
	}()

	request := []byte(`{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"go-test","version":"0.0.0"}}}`)
	if _, err := fmt.Fprintf(stdinWriter, "Content-Length: %d\r\n\r\n%s", len(request), request); err != nil {
		t.Fatalf("write initialize request: %v", err)
	}
	if err := stdinWriter.Close(); err != nil {
		t.Fatalf("close stdin pipe: %v", err)
	}

	body, err := readMCPMessage(bufio.NewReader(stdoutReader), &stderr)
	if err != nil {
		t.Fatalf("read initialize response: %v; stderr=%q", err, stderr.String())
	}

	var response struct {
		Result struct {
			ProtocolVersion string `json:"protocolVersion"`
			ServerInfo      struct {
				Name string `json:"name"`
			} `json:"serverInfo"`
			Capabilities map[string]json.RawMessage `json:"capabilities"`
		} `json:"result"`
		Error *jsonRPCError `json:"error"`
	}
	if err := json.Unmarshal(body, &response); err != nil {
		t.Fatalf("decode initialize response %q: %v", string(body), err)
	}
	if response.Error != nil {
		t.Fatalf("initialize returned error: %+v", response.Error)
	}
	if response.Result.ProtocolVersion == "" {
		t.Fatalf("initialize response missing protocolVersion: %s", string(body))
	}
	if response.Result.ServerInfo.Name != "ai-slop-cleaner" {
		t.Fatalf("serverInfo.name = %q, want ai-slop-cleaner", response.Result.ServerInfo.Name)
	}
	if _, ok := response.Result.Capabilities["tools"]; !ok {
		t.Fatalf("initialize response missing capabilities.tools: %s", string(body))
	}

	select {
	case code := <-done:
		if code != 0 {
			t.Fatalf("serveMCP returned %d; stderr=%q", code, stderr.String())
		}
	case <-time.After(2 * time.Second):
		t.Fatalf("serveMCP did not exit after stdin closed; stderr=%q", stderr.String())
	}
}

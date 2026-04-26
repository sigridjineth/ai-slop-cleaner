// ai-slop-cleaner - Minimal Go CLI to detect AI slop markers in prose.
// Usage:
//
//	go run ai-slop-cleaner.go -input draft.md -output report.md
//	cat draft.md | go run ai-slop-cleaner.go -output report.md
//	cat draft.md | go run ai-slop-cleaner.go -score
//	go run ai-slop-cleaner.go serve
package main

import (
	"bufio"
	"encoding/json"
	"flag"
	"fmt"
	"io"
	"math"
	"os"
	"path/filepath"
	"regexp"
	"sort"
	"strconv"
	"strings"
	"unicode"
	"unicode/utf8"
)

// Finding represents a single slop detection.
type Finding struct {
	Line     int    `json:"line"`
	Category string `json:"category"`
	Severity string `json:"severity"`
	Text     string `json:"text"`
	Context  string `json:"context"`
	Suggest  string `json:"suggest,omitempty"`
}

// DocPatterns captures document-level signals.
type DocPatterns struct {
	AvgSentenceLength    float64  `json:"avg_sentence_length"`
	SentenceLengthStdDev float64  `json:"sentence_length_std_dev"`
	RepeatedOpeners      []string `json:"repeated_openers"`
	BulletDensity        string   `json:"bullet_density"`
	HeadingCount         int      `json:"heading_count"`
	BannedWordCount      int      `json:"banned_word_count"`
}

const (
	bannedWordWeight   = 0.25
	structuralWeight   = 0.25
	rhythmWeight       = 0.20
	metaWeight         = 0.15
	markdownWeight     = 0.15
	bannedDensityMax   = 16.0
	structuralMax      = 10.0
	metaDensityMax     = 8.0
	markdownDensityMax = 12.0
)

var (
	bannedWords = map[string]string{
		"delve": "explore", "elucidate": "explain", "underscore": "stress",
		"harness": "use", "leverage": "use", "bolster": "strengthen",
		"foster": "encourage", "showcase": "show", "streamline": "simplify",
		"revolutionize": "transform", "unveil": "reveal", "orchestrate": "arrange",
		"transcend": "go beyond", "exemplify": "show", "augment": "expand",
		"surpass": "exceed", "pinpoint": "identify", "scrutinize": "examine",
		"unravel": "solve", "embark": "start", "navigate": "handle",
		"elevate": "raise", "unlock": "open up", "unleash": "release",
		"dive": "look", "discover": "find", "craft": "make", "illuminate": "clarify",
		"pivotal": "key", "meticulous": "careful", "intricate": "complex",
		"transformative": "major", "groundbreaking": "new", "unparalleled": "unmatched",
		"comprehensive": "thorough", "robust": "strong", "crucial": "vital",
		"notable": "noteworthy", "formidable": "impressive", "nuanced": "subtle",
		"multifaceted": "varied", "paramount": "top", "instrumental": "helpful",
		"foundational": "basic", "commendable": "admirable", "cutting-edge": "latest",
		"seamless": "smooth", "vibrant": "lively", "bustling": "busy",
		"holistic": "whole", "poised": "ready", "remarkable": "striking",
		"realm": "area", "tapestry": "fabric", "landscape": "field",
		"beacon": "guide", "hurdles": "obstacles", "testament": "proof",
		"game-changer": "breakthrough", "journey": "process", "synergy": "cooperation",
	}

	bannedPhrases = []string{
		"in today's digital age", "it's important to note", "furthermore",
		"moreover", "not only this, but also", "in conclusion", "in closing",
		"let's dive in", "let's explore", "it's worth noting that", "as we navigate",
		"in an era of", "it is essential that", "it should be noted",
		"in order to", "due to the fact that", "with regard to",
		"in the event that", "for the purpose of", "at this point in time",
		"in spite of the fact that", "in the absence of", "it is evident that",
		"it is apparent that", "there is a need for", "a significant number of",
		"a considerable amount of", "it is interesting to note that",
		"one could argue that", "it is possible that", "there is a possibility that",
		"it may be worth considering", "arguably", "in some cases",
		"to a certain extent", "by and large", "in short",
		"put simply", "at the end of the day", "more specifically",
		"to be more precise", "in more detail", "typically",
		"usually", "generally", "roughly", "kind of", "sort of",
	}

	redefinitionPattern = regexp.MustCompile(`(?i)(?:\bnot\s+\w+(?:\s+\w+){0,2}\b|rather\s+than\s+\w+(?:\s+\w+){0,2}|instead\s+of\s+\w+(?:\s+\w+){0,2})\s*[,;:]+\s*\b(?:it\s+is|is|are)\b|\bnot\s+\w+(?:\s+\w+){0,2}\s+but\s+\w+`)
	metaPattern         = regexp.MustCompile(`(?i)(in this section|by now you should|the key takeaway|here's what you need|let's break this|let's take a closer look|we'll examine|one by one|step by step|let's unpack|let's dig deeper|이 장에서는|이번 장에서는|이 글에서는|정리하면|요약하면|핵심은)`)
	wordPattern         = regexp.MustCompile(`[\p{L}\p{N}]+(?:['’][\p{L}\p{N}]+)*`)
	sentenceEndPattern  = regexp.MustCompile(`[.!?]+(?:\s+|$)`)
	bulletLinePattern   = regexp.MustCompile(`^\s*(?:[-*+]\s+|\d+[.)]\s+)`)
	headingPattern      = regexp.MustCompile(`(?m)^\s{0,3}#{1,6}\s+`)
	boldPattern         = regexp.MustCompile(`(?:\*\*|__)[^*_][^\n]*?(?:\*\*|__)`)
	tableLinePattern    = regexp.MustCompile(`(?m)^\s*\|.+\|\s*$`)
	orderedWordPattern  = regexp.MustCompile(`(?i)\b(first|second|third|fourth|fifth|finally|lastly),`)

	metaPhrases = []string{
		"in this section", "by now you should", "the key takeaway",
		"here's what you need", "let's break this", "let's take a closer look",
		"we'll examine", "one by one", "step by step", "let's unpack",
		"let's dig deeper", "to summarize", "in summary", "in conclusion",
		"이 장에서는", "이번 장에서는", "이 글에서는", "정리하면",
		"요약하면", "핵심은", "결론적으로", "다시 말해",
	}
)

func main() {
	os.Exit(run(os.Args[1:], os.Stdin, os.Stdout, os.Stderr))
}

func run(args []string, stdin io.Reader, stdout io.Writer, stderr io.Writer) int {
	if len(args) > 0 && args[0] == "serve" {
		return serveMCP(stdin, stdout, stderr)
	}

	fs := flag.NewFlagSet("ai-slop-cleaner", flag.ContinueOnError)
	fs.SetOutput(stderr)
	inputPath := fs.String("input", "", "Input file path (default: stdin)")
	outputPath := fs.String("output", "", "Output report path (default: stdout)")
	scoreMode := fs.Bool("score", false, "Print AI slop score 0-100 and exit")
	if err := fs.Parse(args); err != nil {
		return 2
	}

	var reader io.Reader = stdin
	if *inputPath != "" {
		f, err := os.Open(*inputPath)
		if err != nil {
			fmt.Fprintf(stderr, "open input: %v\n", err)
			return 1
		}
		defer f.Close()
		reader = f
	}

	data, err := io.ReadAll(reader)
	if err != nil {
		fmt.Fprintf(stderr, "read input: %v\n", err)
		return 1
	}
	text := string(data)

	if *scoreMode {
		fmt.Fprintln(stdout, scoreSlop(text))
		return 0
	}

	ignorePatterns := loadSlopIgnore()
	findings := detect(text, ignorePatterns)
	patterns := analyzeDoc(text)

	report := generateReport(findings, patterns, *inputPath)

	if *outputPath != "" {
		if err := os.WriteFile(*outputPath, []byte(report), 0644); err != nil {
			fmt.Fprintf(stderr, "write output: %v\n", err)
			return 1
		}
		fmt.Fprintf(stderr, "Report written to %s\n", *outputPath)
	} else {
		fmt.Fprintln(stdout, report)
	}
	return 0
}

const mcpProtocolVersion = "2024-11-05"

type jsonRPCMessage struct {
	JSONRPC string          `json:"jsonrpc"`
	ID      json.RawMessage `json:"id,omitempty"`
	Method  string          `json:"method,omitempty"`
	Params  json.RawMessage `json:"params,omitempty"`
}

type jsonRPCResponse struct {
	JSONRPC string          `json:"jsonrpc"`
	ID      json.RawMessage `json:"id"`
	Result  any             `json:"result,omitempty"`
	Error   *jsonRPCError   `json:"error,omitempty"`
}

type jsonRPCError struct {
	Code    int    `json:"code"`
	Message string `json:"message"`
	Data    any    `json:"data,omitempty"`
}

type mcpTextContent struct {
	Type string `json:"type"`
	Text string `json:"text"`
}

type mcpToolResult struct {
	Content []mcpTextContent `json:"content"`
}

type mcpToolCallParams struct {
	Name      string          `json:"name"`
	Arguments json.RawMessage `json:"arguments"`
}

type slopToolArguments struct {
	Text string `json:"text"`
	File string `json:"file"`
}

// serveMCP runs a minimal MCP server over stdio using JSON-RPC 2.0 framing.
// The transport uses Content-Length headers so stdout remains protocol-only;
// all diagnostics and message logs go to stderr.
func serveMCP(stdin io.Reader, stdout io.Writer, stderr io.Writer) int {
	reader := bufio.NewReader(stdin)
	for {
		body, err := readMCPMessage(reader, stderr)
		if err == io.EOF {
			return 0
		}
		if err != nil {
			fmt.Fprintf(stderr, "mcp read error: %v\n", err)
			return 1
		}

		var msg jsonRPCMessage
		if err := json.Unmarshal(body, &msg); err != nil {
			fmt.Fprintf(stderr, "mcp recv: parse-error bytes=%d\n", len(body))
			resp := jsonRPCResponse{
				JSONRPC: "2.0",
				ID:      json.RawMessage("null"),
				Error:   &jsonRPCError{Code: -32700, Message: "Parse error"},
			}
			if err := writeMCPResponse(stdout, stderr, resp); err != nil {
				fmt.Fprintf(stderr, "mcp write error: %v\n", err)
				return 1
			}
			continue
		}

		if msg.JSONRPC != "2.0" || msg.Method == "" {
			if len(msg.ID) > 0 {
				resp := errorResponse(msg.ID, -32600, "Invalid Request", nil)
				if err := writeMCPResponse(stdout, stderr, resp); err != nil {
					fmt.Fprintf(stderr, "mcp write error: %v\n", err)
					return 1
				}
			}
			continue
		}

		fmt.Fprintf(stderr, "mcp recv: method=%s id=%s\n", msg.Method, rpcIDForLog(msg.ID))

		// Notifications have no id and must not receive a response.
		if len(msg.ID) == 0 {
			if msg.Method == "notifications/initialized" {
				fmt.Fprintln(stderr, "mcp notification: initialized")
			}
			continue
		}

		resp := handleMCPRequest(msg)
		if err := writeMCPResponse(stdout, stderr, resp); err != nil {
			fmt.Fprintf(stderr, "mcp write error: %v\n", err)
			return 1
		}
	}
}

func handleMCPRequest(msg jsonRPCMessage) jsonRPCResponse {
	switch msg.Method {
	case "initialize":
		return resultResponse(msg.ID, map[string]any{
			"protocolVersion": mcpProtocolVersion,
			"serverInfo": map[string]any{
				"name":    "ai-slop-cleaner",
				"version": "1.1.0",
			},
			"capabilities": map[string]any{
				"tools": map[string]any{},
			},
		})
	case "tools/list":
		return resultResponse(msg.ID, map[string]any{
			"tools": []map[string]any{
				{
					"name":        "ai_slop_score",
					"description": "Return the 0-100 AI slop score with BWD, SPV, RHY, META, and MD component breakdown.",
					"inputSchema": textOrFileInputSchema(),
				},
				{
					"name":        "ai_slop_analyze",
					"description": "Return a full line-by-line AI slop findings report with categories, severity, and suggestions.",
					"inputSchema": textOrFileInputSchema(),
				},
				{
					"name":        "ai_slop_check",
					"description": "Return a quick pass/fail for inline text. Passing means score <= 25.",
					"inputSchema": map[string]any{
						"type":                 "object",
						"additionalProperties": false,
						"properties": map[string]any{
							"text": map[string]any{"type": "string", "description": "Text to score."},
						},
						"required": []string{"text"},
					},
				},
			},
		})
	case "tools/call":
		result, err := callMCPTool(msg.Params)
		if err != nil {
			return errorResponse(msg.ID, -32602, "Invalid params", err.Error())
		}
		return resultResponse(msg.ID, result)
	default:
		return errorResponse(msg.ID, -32601, "Method not found", msg.Method)
	}
}

func callMCPTool(rawParams json.RawMessage) (mcpToolResult, error) {
	var params mcpToolCallParams
	if len(rawParams) == 0 {
		return mcpToolResult{}, fmt.Errorf("tools/call requires params")
	}
	if err := json.Unmarshal(rawParams, &params); err != nil {
		return mcpToolResult{}, fmt.Errorf("decode tools/call params: %w", err)
	}
	if params.Name == "" {
		return mcpToolResult{}, fmt.Errorf("missing tool name")
	}

	var args slopToolArguments
	if len(params.Arguments) > 0 {
		if err := json.Unmarshal(params.Arguments, &args); err != nil {
			return mcpToolResult{}, fmt.Errorf("decode tool arguments: %w", err)
		}
	}

	switch params.Name {
	case "ai_slop_score":
		text, _, err := textFromToolArguments(args)
		if err != nil {
			return mcpToolResult{}, err
		}
		return textToolResult(scoreToolOutput(text))
	case "ai_slop_analyze":
		text, source, err := textFromToolArguments(args)
		if err != nil {
			return mcpToolResult{}, err
		}
		return textToolResult(analyzeToolOutput(text, source))
	case "ai_slop_check":
		if args.Text == "" {
			return mcpToolResult{}, fmt.Errorf("ai_slop_check requires text")
		}
		return textToolResult(checkToolOutput(args.Text))
	default:
		return mcpToolResult{}, fmt.Errorf("unknown tool %q", params.Name)
	}
}

func textOrFileInputSchema() map[string]any {
	return map[string]any{
		"type":                 "object",
		"additionalProperties": false,
		"properties": map[string]any{
			"text": map[string]any{"type": "string", "description": "Text to analyze."},
			"file": map[string]any{"type": "string", "description": "Path to a UTF-8 text file to analyze."},
		},
		"anyOf": []map[string]any{
			{"required": []string{"text"}},
			{"required": []string{"file"}},
		},
	}
}

func textFromToolArguments(args slopToolArguments) (string, string, error) {
	if args.Text != "" {
		return args.Text, "", nil
	}
	if args.File == "" {
		return "", "", fmt.Errorf("tool requires text or file")
	}
	data, err := os.ReadFile(args.File)
	if err != nil {
		return "", "", fmt.Errorf("read file: %w", err)
	}
	return string(data), args.File, nil
}

func scoreToolOutput(text string) string {
	breakdown := calculateSlopScore(text)
	output := map[string]any{
		"score": breakdown.Score,
		"components": map[string]float64{
			"BWD":  breakdown.BannedWordDensity,
			"SPV":  breakdown.StructuralPatternViol,
			"RHY":  breakdown.RhythmMonotony,
			"META": breakdown.MetaCommentaryDensity,
			"MD":   breakdown.MarkdownOveruse,
		},
		"details": map[string]any{
			"word_count":               breakdown.WordCount,
			"sentence_count":           breakdown.SentenceCount,
			"weighted_banned_hits":     breakdown.WeightedBannedHits,
			"weighted_structural_hits": breakdown.WeightedStructuralHits,
			"meta_hits":                breakdown.MetaHits,
			"markdown_hits":            breakdown.MarkdownHits,
			"rhythm_cv":                breakdown.RhythmCV,
			"rhythm_sample_confidence": breakdown.RhythmSampleConfidence,
		},
	}
	return mustMarshalIndent(output)
}

func analyzeToolOutput(text string, source string) string {
	ignorePatterns := loadSlopIgnore()
	findings := detect(text, ignorePatterns)
	patterns := analyzeDoc(text)
	report := generateReport(findings, patterns, source)
	score := calculateSlopScore(text)

	var b strings.Builder
	b.WriteString(report)
	b.WriteString("\n## AI Slop Score\n\n")
	b.WriteString(fmt.Sprintf("- **Score:** %d/100\n", score.Score))
	b.WriteString(fmt.Sprintf("- **BWD:** %.4f\n", score.BannedWordDensity))
	b.WriteString(fmt.Sprintf("- **SPV:** %.4f\n", score.StructuralPatternViol))
	b.WriteString(fmt.Sprintf("- **RHY:** %.4f\n", score.RhythmMonotony))
	b.WriteString(fmt.Sprintf("- **META:** %.4f\n", score.MetaCommentaryDensity))
	b.WriteString(fmt.Sprintf("- **MD:** %.4f\n", score.MarkdownOveruse))
	return b.String()
}

func checkToolOutput(text string) string {
	score := scoreSlop(text)
	status := "pass"
	if score > 25 {
		status = "fail"
	}
	return mustMarshalIndent(map[string]any{
		"status": status,
		"pass":   score <= 25,
		"score":  score,
	})
}

func textToolResult(text string) (mcpToolResult, error) {
	return mcpToolResult{
		Content: []mcpTextContent{{Type: "text", Text: text}},
	}, nil
}

func resultResponse(id json.RawMessage, result any) jsonRPCResponse {
	return jsonRPCResponse{JSONRPC: "2.0", ID: cloneRawMessage(id), Result: result}
}

func errorResponse(id json.RawMessage, code int, message string, data any) jsonRPCResponse {
	return jsonRPCResponse{
		JSONRPC: "2.0",
		ID:      cloneRawMessage(id),
		Error:   &jsonRPCError{Code: code, Message: message, Data: data},
	}
}

func cloneRawMessage(raw json.RawMessage) json.RawMessage {
	if len(raw) == 0 {
		return json.RawMessage("null")
	}
	cloned := make([]byte, len(raw))
	copy(cloned, raw)
	return cloned
}

func readMCPMessage(reader *bufio.Reader, stderr io.Writer) ([]byte, error) {
	contentLength := -1
	for {
		line, err := reader.ReadString('\n')
		if err != nil {
			if err == io.EOF && line == "" {
				return nil, io.EOF
			}
			if err != io.EOF {
				return nil, err
			}
		}

		trimmed := strings.TrimRight(line, "\r\n")
		if trimmed == "" {
			if contentLength >= 0 {
				break
			}
			if err == io.EOF {
				return nil, io.EOF
			}
			continue
		}

		name, value, ok := strings.Cut(trimmed, ":")
		if !ok {
			fmt.Fprintf(stderr, "mcp ignored non-header input: %q\n", trimmed)
			if err == io.EOF {
				return nil, io.EOF
			}
			continue
		}

		if strings.EqualFold(strings.TrimSpace(name), "Content-Length") {
			n, parseErr := strconv.Atoi(strings.TrimSpace(value))
			if parseErr != nil || n < 0 {
				return nil, fmt.Errorf("invalid Content-Length %q", value)
			}
			contentLength = n
		}

		if err == io.EOF {
			return nil, io.EOF
		}
	}

	body := make([]byte, contentLength)
	if _, err := io.ReadFull(reader, body); err != nil {
		return nil, err
	}
	return body, nil
}

func writeMCPResponse(stdout io.Writer, stderr io.Writer, resp jsonRPCResponse) error {
	body, err := json.Marshal(resp)
	if err != nil {
		return err
	}
	fmt.Fprintf(stderr, "mcp send: id=%s bytes=%d\n", rpcIDForLog(resp.ID), len(body))
	_, err = fmt.Fprintf(stdout, "Content-Length: %d\r\n\r\n%s", len(body), body)
	return err
}

func rpcIDForLog(id json.RawMessage) string {
	if len(id) == 0 {
		return "<notification>"
	}
	return string(id)
}

func mustMarshalIndent(value any) string {
	data, err := json.MarshalIndent(value, "", "  ")
	if err != nil {
		return fmt.Sprintf(`{"error":"marshal output: %v"}`, err)
	}
	return string(data)
}

func loadSlopIgnore() []*regexp.Regexp {
	var patterns []*regexp.Regexp
	paths := []string{".slopignore", os.ExpandEnv("$HOME/.slopignore")}
	for _, p := range paths {
		data, err := os.ReadFile(p)
		if err != nil {
			continue
		}
		lines := strings.Split(string(data), "\n")
		for _, line := range lines {
			line = strings.TrimSpace(line)
			if line == "" || strings.HasPrefix(line, "#") {
				continue
			}
			re, err := regexp.Compile(line)
			if err == nil {
				patterns = append(patterns, re)
			}
		}
	}
	return patterns
}

func shouldIgnore(line string, patterns []*regexp.Regexp) bool {
	for _, re := range patterns {
		if re.MatchString(line) {
			return true
		}
	}
	return false
}

type scoreBreakdown struct {
	Score                  int
	WordCount              int
	SentenceCount          int
	BannedWordDensity      float64
	StructuralPatternViol  float64
	RhythmMonotony         float64
	MetaCommentaryDensity  float64
	MarkdownOveruse        float64
	WeightedBannedHits     int
	WeightedStructuralHits float64
	MetaHits               int
	MarkdownHits           int
	RhythmCV               float64
	RhythmSampleConfidence string
}

type scoreDocument struct {
	Original        string
	CleanText       string
	ProseLines      []string
	Sentences       []string
	WordCount       int
	InlineCodeSpans int
}

type bannedCatalog struct {
	SingleWords []string
	Phrases     []string
}

// scoreSlop implements references/slop-score-spec.md's weighted 0-100 score.
// The direct refactor request resolves two draft-spec ambiguities for -score:
// META normalizes by 8 hits/1,000 words and MD uses a simple markdown-hit
// density over headers, bullets, and bold spans normalized by 12.
func scoreSlop(text string) int {
	return calculateSlopScore(text).Score
}

func calculateSlopScore(text string) scoreBreakdown {
	ignorePatterns := loadSlopIgnore()
	doc := prepareScoreDocument(text, ignorePatterns)
	bwd, weightedBannedHits := bannedWordDensitySubscore(doc)
	spv, weightedStructuralHits := structuralPatternSubscore(doc)
	rhy, cv, confidence := rhythmMonotonySubscore(doc)
	meta, metaHits := metaCommentarySubscore(doc)
	md, markdownHits := markdownOveruseSubscore(doc)

	return scoreBreakdown{
		Score:                  scoreFromComponents(bwd, spv, rhy, meta, md),
		WordCount:              doc.WordCount,
		SentenceCount:          len(doc.Sentences),
		BannedWordDensity:      round4(bwd),
		StructuralPatternViol:  round4(spv),
		RhythmMonotony:         round4(rhy),
		MetaCommentaryDensity:  round4(meta),
		MarkdownOveruse:        round4(md),
		WeightedBannedHits:     weightedBannedHits,
		WeightedStructuralHits: round4(weightedStructuralHits),
		MetaHits:               metaHits,
		MarkdownHits:           markdownHits,
		RhythmCV:               round4(cv),
		RhythmSampleConfidence: confidence,
	}
}

func scoreFromComponents(bwd, spv, rhy, meta, md float64) int {
	score := 100 * ((bannedWordWeight * clip01(bwd)) +
		(structuralWeight * clip01(spv)) +
		(rhythmWeight * clip01(rhy)) +
		(metaWeight * clip01(meta)) +
		(markdownWeight * clip01(md)))
	if math.IsNaN(score) || score < 0 {
		return 0
	}
	if score > 100 {
		return 100
	}
	return int(math.Round(score))
}

func prepareScoreDocument(text string, ignorePatterns []*regexp.Regexp) scoreDocument {
	withoutFrontMatter := removeFrontMatter(text)
	lines := strings.Split(withoutFrontMatter, "\n")
	inFence := false
	var cleanLines []string
	var proseLines []string
	inlineCodeSpans := 0

	for _, line := range lines {
		trimmed := strings.TrimSpace(line)
		if fenceMarker(trimmed) != "" {
			inFence = !inFence
			continue
		}
		if inFence {
			continue
		}
		if shouldIgnore(line, ignorePatterns) {
			continue
		}

		stripped, spans := stripInlineCode(line)
		inlineCodeSpans += spans
		cleanLines = append(cleanLines, stripped)
		if strings.TrimSpace(stripped) != "" {
			proseLines = append(proseLines, stripped)
		}
	}

	cleanText := strings.Join(cleanLines, "\n")
	sentences := splitSentencesForScore(cleanText)
	return scoreDocument{
		Original:        text,
		CleanText:       cleanText,
		ProseLines:      proseLines,
		Sentences:       sentences,
		WordCount:       maxInt(len(wordTokens(cleanText)), 1),
		InlineCodeSpans: inlineCodeSpans,
	}
}

func removeFrontMatter(text string) string {
	lines := strings.Split(text, "\n")
	if len(lines) == 0 {
		return text
	}
	first := strings.TrimSpace(lines[0])
	if first != "---" && first != "+++" {
		return text
	}
	for i := 1; i < len(lines); i++ {
		if strings.TrimSpace(lines[i]) == first {
			return strings.Join(lines[i+1:], "\n")
		}
	}
	return text
}

func fenceMarker(trimmed string) string {
	if strings.HasPrefix(trimmed, "```") {
		return "```"
	}
	if strings.HasPrefix(trimmed, "~~~") {
		return "~~~"
	}
	return ""
}

func stripInlineCode(line string) (string, int) {
	spans := regexp.MustCompile("`[^`\n]*`").FindAllStringIndex(line, -1)
	if len(spans) == 0 {
		return line, 0
	}
	var b strings.Builder
	last := 0
	for _, span := range spans {
		b.WriteString(line[last:span[0]])
		b.WriteString(strings.Repeat(" ", span[1]-span[0]))
		last = span[1]
	}
	b.WriteString(line[last:])
	return b.String(), len(spans)
}

func bannedWordDensitySubscore(doc scoreDocument) (float64, int) {
	catalog := loadBannedCatalog()
	phraseSpans := make([][2]int, 0)
	phraseHits := 0
	for _, phrase := range catalog.Phrases {
		re := normalizedPhraseRegexp(phrase)
		matches := re.FindAllStringSubmatchIndex(doc.CleanText, -1)
		phraseHits += len(matches)
		for _, m := range matches {
			// Narrow span to exclude boundary capture groups so single-word
			// matches adjacent to a phrase are not falsely excluded.
			pStart := m[3] // end of leading boundary group
			pEnd := m[4]   // start of trailing boundary group
			if pStart < 0 {
				pStart = m[0]
			}
			if pEnd < 0 {
				pEnd = m[1]
			}
			phraseSpans = append(phraseSpans, [2]int{pStart, pEnd})
		}
	}

	singleHits := 0
	for _, word := range catalog.SingleWords {
		re := wholeEntryRegexp(word)
		for _, m := range re.FindAllStringIndex(doc.CleanText, -1) {
			if !spanOverlapsAny(m[0], m[1], phraseSpans) {
				singleHits++
			}
		}
	}

	weightedHits := singleHits + (2 * phraseHits)
	density := 1000 * float64(weightedHits) / float64(doc.WordCount)
	return clip01(density / bannedDensityMax), weightedHits
}

func structuralPatternSubscore(doc scoreDocument) (float64, float64) {
	weightedHits := 0.0
	weightedHits += 2.0 * float64(len(redefinitionPattern.FindAllString(doc.CleanText, -1)))
	weightedHits += 2.0 * float64(countSentenceTemplateStreaks(doc.Sentences))
	if hasRepeatedSectionMold(doc.ProseLines) {
		weightedHits += 2.0
	}
	weightedHits += 1.5 * float64(countOverNumberedSequences(doc.CleanText, doc.ProseLines))
	weightedHits += 1.5 * float64(countBroadOverviewFieldDumps(doc.CleanText))
	weightedHits += 1.0 * float64(countClosingSummaryRepetition(doc.CleanText))
	weightedHits += 1.0 * float64(countConnectorMonotony(doc.CleanText))
	weightedHits += 1.0 * float64(countProgressAnnouncements(doc.CleanText))
	weightedHits += 1.0 * float64(countPreClassificationFraming(doc.CleanText))
	weightedHits += 1.0 * float64(countImaginePrompts(doc.CleanText))
	weightedHits += 1.0 * float64(countPostCodeTextbookNarration(doc.Original))

	density := 1000 * weightedHits / float64(doc.WordCount)
	return clip01(density / structuralMax), weightedHits
}

func rhythmMonotonySubscore(doc scoreDocument) (float64, float64, string) {
	if len(doc.Sentences) < 5 {
		return 0, 0, "low"
	}

	lengths := make([]int, 0, len(doc.Sentences))
	for _, sentence := range doc.Sentences {
		if l := len(wordTokens(sentence)); l > 0 {
			lengths = append(lengths, l)
		}
	}
	if len(lengths) < 5 {
		return 0, 0, "low"
	}

	mean, sd := averageAndStdDev(lengths)
	cv := sd / math.Max(mean, 1)
	cvPenalty := clip01((0.55 - cv) / 0.35)
	if mean < 8 {
		cvPenalty *= 0.5
	}

	openerRepeatRate := repeatedOpenerRate(doc.Sentences)
	openerPenalty := clip01((openerRepeatRate - 0.15) / 0.25)
	endingRepeatRate := repeatedEndingRate(doc.Sentences)
	endingPenalty := clip01((endingRepeatRate - 0.15) / 0.25)
	maxStreak := maxSentenceTemplateStreak(doc.Sentences)
	templateStreakPenalty := clip01((float64(maxStreak) - 2) / 4)

	rhy := (0.50 * cvPenalty) +
		(0.20 * openerPenalty) +
		(0.20 * endingPenalty) +
		(0.10 * templateStreakPenalty)
	return clip01(rhy), cv, "normal"
}

func metaCommentarySubscore(doc scoreDocument) (float64, int) {
	hits := countMetaCommentaryHits(doc.CleanText, doc.WordCount)
	density := 1000 * float64(hits) / float64(doc.WordCount)
	return clip01(density / metaDensityMax), hits
}

func markdownOveruseSubscore(doc scoreDocument) (float64, int) {
	hits := 0
	for _, line := range doc.ProseLines {
		if headingLinePattern.MatchString(line) {
			hits++
		}
		if bulletLinePattern.MatchString(line) {
			hits++
		}
		hits += len(boldPattern.FindAllString(line, -1))
	}
	density := 1000 * float64(hits) / float64(doc.WordCount)
	return clip01(density / markdownDensityMax), hits
}

func loadBannedCatalog() bannedCatalog {
	if data, err := readReferenceFile("banned-words.md"); err == nil {
		if catalog := parseBannedCatalog(string(data)); len(catalog.SingleWords)+len(catalog.Phrases) > 0 {
			return catalog
		}
	}

	catalog := bannedCatalog{}
	for word := range bannedWords {
		catalog.SingleWords = append(catalog.SingleWords, word)
	}
	catalog.Phrases = append(catalog.Phrases, bannedPhrases...)
	sort.Strings(catalog.SingleWords)
	sort.Strings(catalog.Phrases)
	return catalog
}

func readReferenceFile(name string) ([]byte, error) {
	seen := make(map[string]bool)
	var candidates []string
	if cwd, err := os.Getwd(); err == nil {
		for dir := cwd; ; dir = filepath.Dir(dir) {
			candidates = append(candidates, filepath.Join(dir, "references", name))
			parent := filepath.Dir(dir)
			if parent == dir {
				break
			}
		}
	}
	if exe, err := os.Executable(); err == nil {
		exeDir := filepath.Dir(exe)
		candidates = append(candidates,
			filepath.Join(exeDir, "references", name),
			filepath.Join(exeDir, "..", "references", name),
		)
	}

	var lastErr error
	for _, candidate := range candidates {
		clean := filepath.Clean(candidate)
		if seen[clean] {
			continue
		}
		seen[clean] = true
		data, err := os.ReadFile(clean)
		if err == nil {
			return data, nil
		}
		lastErr = err
	}
	if lastErr == nil {
		lastErr = os.ErrNotExist
	}
	return nil, lastErr
}

func parseBannedCatalog(markdown string) bannedCatalog {
	singles := make(map[string]bool)
	phrases := make(map[string]bool)
	add := func(raw string) {
		entry := normalizeCatalogEntry(raw)
		if entry == "" || entry == "banned" || strings.Contains(entry, "---") {
			return
		}
		if strings.ContainsAny(entry, " \t\n\r") {
			phrases[entry] = true
		} else {
			singles[entry] = true
		}
	}

	for _, line := range strings.Split(markdown, "\n") {
		trimmed := strings.TrimSpace(line)
		if strings.HasPrefix(trimmed, "|") && strings.Count(trimmed, "|") >= 2 {
			cols := strings.Split(trimmed, "|")
			if len(cols) >= 3 {
				cell := strings.TrimSpace(cols[1])
				if cell != "" && !strings.Contains(cell, "---") && !strings.EqualFold(cell, "Banned") && !strings.EqualFold(cell, "Stiff") {
					for _, part := range strings.Split(cell, ",") {
						add(part)
					}
				}
			}
			continue
		}

		if strings.HasPrefix(trimmed, "- ") {
			if quoted := firstQuoted(trimmed); quoted != "" {
				add(quoted)
			}
		}
	}

	catalog := bannedCatalog{
		SingleWords: mapKeys(singles),
		Phrases:     mapKeys(phrases),
	}
	sort.Strings(catalog.SingleWords)
	sort.Strings(catalog.Phrases)
	return catalog
}

func normalizeCatalogEntry(raw string) string {
	entry := strings.TrimSpace(raw)
	entry = strings.Trim(entry, "`*_ ")
	entry = strings.Trim(entry, "\"“”")
	entry = regexp.MustCompile(`\s+\([^)]*\)\s*$`).ReplaceAllString(entry, "")
	entry = strings.TrimSpace(entry)
	entry = strings.Trim(entry, ".;,:")
	entry = regexp.MustCompile(`\s+`).ReplaceAllString(entry, " ")
	return strings.ToLower(entry)
}

func firstQuoted(line string) string {
	for _, quote := range []string{"\"", "“"} {
		start := strings.Index(line, quote)
		if start < 0 {
			continue
		}
		endQuote := quote
		if quote == "“" {
			endQuote = "”"
		}
		end := strings.Index(line[start+len(quote):], endQuote)
		if end >= 0 {
			return line[start+len(quote) : start+len(quote)+end]
		}
	}
	return ""
}

func normalizedPhraseRegexp(phrase string) *regexp.Regexp {
	fields := strings.Fields(phrase)
	parts := make([]string, 0, len(fields))
	for _, field := range fields {
		parts = append(parts, regexp.QuoteMeta(field))
	}
	return regexp.MustCompile(`(?i)(^|[^\p{L}\p{N}'’])` + strings.Join(parts, `\s+`) + `([^\p{L}\p{N}'’]|$)`)
}

func wholeEntryRegexp(entry string) *regexp.Regexp {
	return regexp.MustCompile(`(?i)(^|[^\p{L}\p{N}'’])` + regexp.QuoteMeta(entry) + `([^\p{L}\p{N}'’]|$)`)
}

func spanOverlapsAny(start, end int, spans [][2]int) bool {
	for _, span := range spans {
		if start < span[1] && end > span[0] {
			return true
		}
	}
	return false
}

func splitSentencesForScore(text string) []string {
	var sentences []string
	for _, paragraph := range regexp.MustCompile(`\n\s*\n+`).Split(text, -1) {
		paragraph = strings.TrimSpace(paragraph)
		if paragraph == "" {
			continue
		}
		start := 0
		for i, r := range paragraph {
			if r == '.' || r == '!' || r == '?' {
				part := strings.TrimSpace(paragraph[start:i])
				if part != "" {
					sentences = append(sentences, part)
				}
				start = i + len(string(r))
			}
		}
		if tail := strings.TrimSpace(paragraph[start:]); tail != "" {
			sentences = append(sentences, tail)
		}
	}
	return sentences
}

func wordTokens(text string) []string {
	return wordPattern.FindAllString(text, -1)
}

func countSentenceTemplateStreaks(sentences []string) int {
	streaks := 0
	last := ""
	run := 0
	for _, sentence := range sentences {
		skeleton := sentenceSkeleton(sentence)
		if skeleton != "" && skeleton == last {
			run++
		} else {
			if run >= 3 {
				streaks++
			}
			last = skeleton
			if skeleton == "" {
				run = 0
			} else {
				run = 1
			}
		}
	}
	if run >= 3 {
		streaks++
	}
	return streaks
}

func maxSentenceTemplateStreak(sentences []string) int {
	maxStreak := 0
	last := ""
	run := 0
	for _, sentence := range sentences {
		skeleton := sentenceSkeleton(sentence)
		if skeleton != "" && skeleton == last {
			run++
		} else {
			if run > maxStreak {
				maxStreak = run
			}
			last = skeleton
			if skeleton == "" {
				run = 0
			} else {
				run = 1
			}
		}
	}
	if run > maxStreak {
		maxStreak = run
	}
	return maxStreak
}

func sentenceSkeleton(sentence string) string {
	tokens := wordTokens(strings.ToLower(sentence))
	if len(tokens) == 0 {
		return ""
	}
	limit := len(tokens)
	if limit > 8 {
		limit = 8
	}
	classes := make([]string, 0, limit)
	for _, token := range tokens[:limit] {
		classes = append(classes, coarseTokenClass(token))
	}
	return strings.Join(classes, "-")
}

func coarseTokenClass(token string) string {
	switch {
	case isNumberToken(token):
		return "number"
	case inSet(token, pronouns):
		return "pronoun"
	case inSet(token, determiners):
		return "determiner"
	case inSet(token, prepositions):
		return "preposition"
	case inSet(token, conjunctions):
		return "conjunction"
	case inSet(token, commonVerbs) || strings.HasSuffix(token, "ed") || strings.HasSuffix(token, "ing"):
		return "verb"
	case inSet(token, commonAdverbs) || strings.HasSuffix(token, "ly"):
		return "adverb"
	case inSet(token, commonAdjectives) || hasAnySuffix(token, []string{"ive", "al", "ous", "ful", "less", "able", "ible", "ic", "ary"}):
		return "adjective"
	default:
		// Without a POS tagger in the standard library, default open-class words to noun.
		return "noun"
	}
}

var (
	pronouns = map[string]bool{
		"i": true, "me": true, "my": true, "mine": true, "you": true, "your": true, "yours": true,
		"he": true, "him": true, "his": true, "she": true, "her": true, "hers": true, "it": true, "its": true,
		"we": true, "us": true, "our": true, "ours": true, "they": true, "them": true, "their": true, "theirs": true,
		"who": true, "whom": true, "whose": true, "which": true, "what": true,
	}
	determiners = map[string]bool{
		"a": true, "an": true, "the": true, "this": true, "that": true, "these": true, "those": true,
		"each": true, "every": true, "some": true, "any": true, "few": true, "many": true, "much": true,
	}
	prepositions = map[string]bool{
		"of": true, "to": true, "in": true, "for": true, "with": true, "on": true, "at": true, "by": true,
		"from": true, "about": true, "as": true, "into": true, "over": true, "after": true, "before": true,
		"between": true, "through": true, "without": true, "within": true, "under": true, "around": true,
	}
	conjunctions = map[string]bool{
		"and": true, "or": true, "but": true, "nor": true, "yet": true, "so": true, "because": true,
		"although": true, "while": true, "if": true, "when": true, "unless": true,
	}
	commonVerbs = map[string]bool{
		"is": true, "are": true, "was": true, "were": true, "be": true, "being": true, "been": true,
		"am": true, "have": true, "has": true, "had": true, "do": true, "does": true, "did": true,
		"make": true, "makes": true, "made": true, "use": true, "uses": true, "used": true, "need": true,
		"needs": true, "contains": true, "includes": true, "shows": true, "show": true, "discusses": true,
		"covers": true, "highlights": true, "matters": true, "means": true,
	}
	commonAdverbs    = map[string]bool{"very": true, "more": true, "most": true, "often": true, "also": true, "then": true}
	commonAdjectives = map[string]bool{"good": true, "bad": true, "new": true, "old": true, "key": true, "same": true, "clear": true}
	functionWords    = map[string]bool{
		"a": true, "an": true, "the": true, "of": true, "to": true, "in": true, "for": true, "with": true,
		"and": true, "or": true, "but": true, "is": true, "are": true, "was": true, "were": true,
	}
)

func hasRepeatedSectionMold(lines []string) bool {
	moldCount := 0
	for i := 0; i < len(lines); i++ {
		if !headingLinePattern.MatchString(lines[i]) {
			continue
		}
		introLines := 0
		bulletLines := 0
		j := i + 1
		for j < len(lines) && strings.TrimSpace(lines[j]) == "" {
			j++
		}
		for j < len(lines) && !bulletLinePattern.MatchString(lines[j]) && !headingLinePattern.MatchString(lines[j]) {
			introLines++
			j++
		}
		for j < len(lines) && bulletLinePattern.MatchString(lines[j]) {
			bulletLines++
			j++
		}
		if introLines >= 1 && introLines <= 2 && bulletLines >= 3 && bulletLines <= 6 {
			moldCount++
		}
	}
	return moldCount >= 3
}

func countOverNumberedSequences(text string, lines []string) int {
	count := 0
	ordinalMatches := regexp.MustCompile(`(?i)\b(first|second|third|fourth|fifth|sixth|finally|lastly)\b`).FindAllStringIndex(text, -1)
	if len(ordinalMatches) >= 3 {
		count++
	}

	run := 0
	for _, line := range lines {
		if regexp.MustCompile(`^\s*\d+[.)]\s+`).MatchString(line) {
			run++
		} else {
			if run >= 3 {
				count++
			}
			run = 0
		}
	}
	if run >= 3 {
		count++
	}
	return count
}

func countBroadOverviewFieldDumps(text string) int {
	patterns := []*regexp.Regexp{
		regexp.MustCompile(`(?is)\b(?:fields|properties|attributes|schema|structure)\b[^.\n]{0,120}:\s*(?:\n\s*(?:[-*+]\s*)?` + "`?" + `[\p{L}_][\p{L}\p{N}_-]*` + "`?" + `\s*:[^\n]*){2,}`),
		regexp.MustCompile(`(?is)\b(?:fields|properties|attributes|schema|structure)\b[^.\n]{0,120}:\s*(?:` + "`?" + `[\p{L}_][\p{L}\p{N}_-]*` + "`?" + `\s*:[^,;.\n]*){2,}`),
	}
	count := 0
	for _, re := range patterns {
		count += len(re.FindAllStringIndex(text, -1))
	}
	return count
}

func countClosingSummaryRepetition(text string) int {
	return len(regexp.MustCompile(`(?i)\b(to summarize|in summary|in conclusion|in closing|in short|put simply|at the end of the day)\b`).FindAllStringIndex(text, -1))
}

func countConnectorMonotony(text string) int {
	counts := make(map[string]int)
	for _, paragraph := range regexp.MustCompile(`\n\s*\n+`).Split(text, -1) {
		tokens := wordTokens(strings.ToLower(paragraph))
		if len(tokens) == 0 {
			continue
		}
		opener := tokens[0]
		if len(tokens) >= 3 {
			candidate := strings.Join(tokens[:3], " ")
			if connectorOpeners[candidate] {
				opener = candidate
			}
		}
		if len(tokens) >= 2 {
			candidate := strings.Join(tokens[:2], " ")
			if connectorOpeners[candidate] {
				opener = candidate
			}
		}
		if connectorOpeners[opener] {
			counts[opener]++
		}
	}
	groups := 0
	for _, count := range counts {
		if count >= 3 {
			groups++
		}
	}
	return groups
}

var connectorOpeners = map[string]bool{
	"furthermore": true, "moreover": true, "however": true, "therefore": true, "consequently": true,
	"additionally": true, "meanwhile": true, "overall": true, "first": true, "second": true, "third": true,
	"as a result": true, "in addition": true, "on the other": true,
}

func countProgressAnnouncements(text string) int {
	return len(regexp.MustCompile(`(?i)\b(let's\s+(?:take a closer look|unpack(?: this)?|dig deeper|break this down|dive in|explore)|we(?:'ll| will)\s+examine(?: this)?(?: step by step)?|step by step|one by one)\b`).FindAllStringIndex(text, -1))
}

func countPreClassificationFraming(text string) int {
	return len(regexp.MustCompile(`(?i)\b(?:there are|there is|we can divide|this divides|can be divided)\b[^.!?]{0,80}\b(?:types|branches|categories|groups|kinds)\b`).FindAllStringIndex(text, -1))
}

func countImaginePrompts(text string) int {
	return len(regexp.MustCompile(`(?i)\b(imagine|picture|visualize)\b[^.!?]{0,80}`).FindAllStringIndex(text, -1))
}

func countPostCodeTextbookNarration(text string) int {
	lines := strings.Split(text, "\n")
	inFence := false
	justClosedFence := false
	count := 0
	narration := regexp.MustCompile(`(?i)\b(first|then|next)\s+(?:we|the code|this)|\bwe\s+(?:define|create|call|return)\b`)
	for _, line := range lines {
		trimmed := strings.TrimSpace(line)
		if fenceMarker(trimmed) != "" {
			if inFence {
				justClosedFence = true
			}
			inFence = !inFence
			continue
		}
		if inFence || strings.TrimSpace(line) == "" {
			continue
		}
		if justClosedFence {
			if narration.MatchString(line) {
				count++
			}
			justClosedFence = false
		}
	}
	return count
}

func repeatedOpenerRate(sentences []string) float64 {
	counts := make(map[string]int)
	sentenceOpeners := make([]string, 0, len(sentences))
	for _, sentence := range sentences {
		tokens := wordTokens(strings.ToLower(sentence))
		if len(tokens) == 0 {
			sentenceOpeners = append(sentenceOpeners, "")
			continue
		}
		opener := tokens[0]
		sentenceOpeners = append(sentenceOpeners, opener)
		counts[opener]++
	}
	repeated := 0
	for _, opener := range sentenceOpeners {
		if opener != "" && counts[opener] >= 3 {
			repeated++
		}
	}
	return float64(repeated) / float64(len(sentences))
}

func repeatedEndingRate(sentences []string) float64 {
	counts := make(map[string]int)
	endings := make([]string, 0, len(sentences))
	for _, sentence := range sentences {
		ending := sentenceEndingKey(sentence)
		endings = append(endings, ending)
		if ending != "" {
			counts[ending]++
		}
	}
	repeated := 0
	for _, ending := range endings {
		if ending != "" && counts[ending] >= 2 {
			repeated++
		}
	}
	return float64(repeated) / float64(len(sentences))
}

func sentenceEndingKey(sentence string) string {
	tokens := wordTokens(strings.ToLower(sentence))
	content := make([]string, 0, len(tokens))
	for _, token := range tokens {
		if !functionWords[token] {
			content = append(content, token)
		}
	}
	if len(content) >= 2 {
		return strings.Join(content[len(content)-2:], " ")
	}
	if len(content) == 1 {
		return content[0]
	}
	if len(tokens) > 0 {
		return tokens[len(tokens)-1]
	}
	return ""
}

var metaRegexps = []*regexp.Regexp{
	regexp.MustCompile(`(?i)\bin this section\b`),
	regexp.MustCompile(`(?i)\blet's\s+(?:dive in|explore|unpack(?: this)?|break this down|take a closer look|dig deeper)\b`),
	regexp.MustCompile(`(?i)\bhere's what you need to know\b`),
	regexp.MustCompile(`(?i)\bthe key takeaway(?: here)? is\b`),
	regexp.MustCompile(`(?i)\bto summarize\b`),
	regexp.MustCompile(`(?i)\bin summary\b`),
	regexp.MustCompile(`(?i)\bin conclusion\b`),
	regexp.MustCompile(`(?i)\bit is important to note\b`),
	regexp.MustCompile(`(?i)\bit's important to note\b`),
	regexp.MustCompile(`(?i)\bit should be noted\b`),
	regexp.MustCompile(`(?i)\bit's worth noting that\b`),
	regexp.MustCompile(`(?i)\bby now you should\b`),
	regexp.MustCompile(`(?i)\bwe(?:'ll| will)\s+examine\b`),
	regexp.MustCompile(`(?i)\b(?:step by step|one by one)\b`),
	regexp.MustCompile(`(?i)(이 장에서는|이번 장에서는|이 글에서는|정리하면|요약하면|핵심은|결론적으로|다시 말해)`),
}

func countMetaCommentaryHits(text string, wordCount int) int {
	total := 0
	firstStart := -1
	for _, re := range metaRegexps {
		matches := re.FindAllStringIndex(text, -1)
		for _, m := range matches {
			if firstStart == -1 || m[0] < firstStart {
				firstStart = m[0]
			}
		}
		total += len(matches)
	}
	// Spec allows one useful roadmap at the very top of long technical docs.
	// With no genre metadata in -score mode, approximate "top" as the first 500 bytes.
	if total == 1 && wordCount >= 800 && firstStart >= 0 && firstStart <= 500 {
		return 0
	}
	return total
}

var headingLinePattern = regexp.MustCompile(`^\s{0,3}#{1,6}\s+`)

func clip01(value float64) float64 {
	return clampFloat(value, 0, 1)
}

func round4(value float64) float64 {
	return math.Round(value*10000) / 10000
}

func mapKeys(m map[string]bool) []string {
	keys := make([]string, 0, len(m))
	for key := range m {
		keys = append(keys, key)
	}
	return keys
}

func maxInt(a, b int) int {
	if a > b {
		return a
	}
	return b
}

func inSet(value string, set map[string]bool) bool {
	return set[value]
}

func hasAnySuffix(value string, suffixes []string) bool {
	for _, suffix := range suffixes {
		if strings.HasSuffix(value, suffix) {
			return true
		}
	}
	return false
}

func isNumberToken(value string) bool {
	for _, r := range value {
		if !unicode.IsDigit(r) {
			return false
		}
	}
	return value != ""
}

func detect(text string, ignorePatterns []*regexp.Regexp) []Finding {
	var findings []Finding
	lines := strings.Split(text, "\n")
	for i, line := range lines {
		lineNum := i + 1
		if shouldIgnore(line, ignorePatterns) {
			continue
		}
		lower := strings.ToLower(line)

		// Banned words
		for word, suggest := range bannedWords {
			re := regexp.MustCompile(`\b` + regexp.QuoteMeta(word) + `\b`)
			for _, idx := range re.FindAllStringIndex(lower, -1) {
				findings = append(findings, Finding{
					Line:     lineNum,
					Category: "banned_word",
					Severity: "high",
					Text:     line[idx[0]:idx[1]],
					Context:  strings.TrimSpace(line),
					Suggest:  suggest,
				})
			}
		}

		// Banned phrases
		for _, phrase := range bannedPhrases {
			if strings.Contains(lower, phrase) {
				findings = append(findings, Finding{
					Line:     lineNum,
					Category: "banned_phrase",
					Severity: "high",
					Text:     phrase,
					Context:  strings.TrimSpace(line),
				})
			}
		}

		// Redefinition pattern
		if redefinitionPattern.MatchString(line) {
			findings = append(findings, Finding{
				Line:     lineNum,
				Category: "redefinition",
				Severity: "medium",
				Text:     "not X, it is Y",
				Context:  strings.TrimSpace(line),
			})
		}

		// Meta commentary
		if metaPattern.MatchString(line) {
			findings = append(findings, Finding{
				Line:     lineNum,
				Category: "meta_commentary",
				Severity: "medium",
				Text:     strings.TrimSpace(line),
				Context:  strings.TrimSpace(line),
			})
		}
	}

	// Sort by line
	sort.Slice(findings, func(i, j int) bool {
		if findings[i].Line == findings[j].Line {
			return findings[i].Category < findings[j].Category
		}
		return findings[i].Line < findings[j].Line
	})

	return findings
}

func countBannedWordHits(lowerText string) int {
	count := 0
	for word := range bannedWords {
		re := regexp.MustCompile(`\b` + regexp.QuoteMeta(word) + `\b`)
		count += len(re.FindAllString(lowerText, -1))
	}
	return count
}

func countPhraseHits(lowerText string, phrases []string) int {
	count := 0
	for _, phrase := range phrases {
		count += strings.Count(lowerText, strings.ToLower(phrase))
	}
	return count
}

func splitSentences(text string) []string {
	normalized := strings.ReplaceAll(text, "\n", " ")
	parts := sentenceEndPattern.Split(normalized, -1)
	var sentences []string
	for _, part := range parts {
		part = strings.TrimSpace(part)
		if part != "" {
			sentences = append(sentences, part)
		}
	}
	return sentences
}

func sentenceSignature(sentence string, words int) string {
	tokens := wordPattern.FindAllString(strings.ToLower(sentence), -1)
	if len(tokens) < words {
		return ""
	}
	return strings.Join(tokens[:words], " ")
}

func sentenceWordLengths(text string) []int {
	sentences := splitSentences(text)
	var lengths []int
	for _, sentence := range sentences {
		words := wordPattern.FindAllString(sentence, -1)
		if len(words) > 0 {
			lengths = append(lengths, len(words))
		}
	}
	return lengths
}

func averageAndStdDev(lengths []int) (float64, float64) {
	if len(lengths) == 0 {
		return 0, 0
	}

	sum := 0
	for _, length := range lengths {
		sum += length
	}
	avg := float64(sum) / float64(len(lengths))

	var sqDiff float64
	for _, length := range lengths {
		diff := float64(length) - avg
		sqDiff += diff * diff
	}
	return avg, math.Sqrt(sqDiff / float64(len(lengths)))
}

func countBulletLines(lines []string) int {
	count := 0
	for _, line := range lines {
		if bulletLinePattern.MatchString(line) {
			count++
		}
	}
	return count
}

func clampFloat(value float64, min float64, max float64) float64 {
	if value < min {
		return min
	}
	if value > max {
		return max
	}
	return value
}

func analyzeDoc(text string) DocPatterns {
	// Sentences (rough split by . ! ?)
	sentences := regexp.MustCompile(`[.!?]+\s+`).Split(text, -1)
	var lengths []int
	for _, s := range sentences {
		s = strings.TrimSpace(s)
		if s == "" {
			continue
		}
		lengths = append(lengths, utf8.RuneCountInString(s))
	}

	var avg, variance float64
	if len(lengths) > 0 {
		sum := 0
		for _, l := range lengths {
			sum += l
		}
		avg = float64(sum) / float64(len(lengths))
		var sqDiff float64
		for _, l := range lengths {
			d := float64(l) - avg
			sqDiff += d * d
		}
		variance = sqDiff / float64(len(lengths))
	}

	stdDev := 0.0
	if variance > 0 {
		stdDev = math.Sqrt(variance)
	}

	// Repeated openers
	openerCounts := make(map[string]int)
	for _, s := range sentences {
		s = strings.TrimSpace(s)
		fields := strings.Fields(s)
		if len(fields) > 0 {
			opener := strings.ToLower(fields[0])
			if len(opener) > 3 {
				openerCounts[opener]++
			}
		}
	}
	var repeated []string
	for w, c := range openerCounts {
		if c >= 3 {
			repeated = append(repeated, fmt.Sprintf("%s (%dx)", w, c))
		}
	}
	sort.Strings(repeated)

	// Bullet density
	lines := strings.Split(text, "\n")
	bulletCount := 0
	for _, line := range lines {
		trimmed := strings.TrimSpace(line)
		if strings.HasPrefix(trimmed, "-") || strings.HasPrefix(trimmed, "*") || regexp.MustCompile(`^\d+\.`).MatchString(trimmed) {
			bulletCount++
		}
	}
	bulletDensity := "low"
	if len(lines) > 0 {
		ratio := float64(bulletCount) / float64(len(lines))
		switch {
		case ratio > 0.3:
			bulletDensity = "high"
		case ratio > 0.15:
			bulletDensity = "medium"
		}
	}

	// Headings
	headingCount := len(regexp.MustCompile(`(?m)^#{1,6}\s+`).FindAllString(text, -1))

	// Banned word count (re-count for stats)
	lowerText := strings.ToLower(text)
	bannedCount := 0
	for word := range bannedWords {
		re := regexp.MustCompile(`\b` + regexp.QuoteMeta(word) + `\b`)
		bannedCount += len(re.FindAllString(lowerText, -1))
	}

	return DocPatterns{
		AvgSentenceLength:    avg,
		SentenceLengthStdDev: stdDev,
		RepeatedOpeners:      repeated,
		BulletDensity:        bulletDensity,
		HeadingCount:         headingCount,
		BannedWordCount:      bannedCount,
	}
}

func generateReport(findings []Finding, patterns DocPatterns, inputPath string) string {
	var b strings.Builder

	b.WriteString("# AI Slop Cleaner Report\n\n")
	if inputPath != "" {
		b.WriteString(fmt.Sprintf("**Source:** `%s`\n\n", inputPath))
	}

	b.WriteString("## Document Patterns\n\n")
	b.WriteString(fmt.Sprintf("- **Average sentence length:** %.1f words\n", patterns.AvgSentenceLength))
	b.WriteString(fmt.Sprintf("- **Sentence length std dev:** %.1f\n", patterns.SentenceLengthStdDev))
	b.WriteString(fmt.Sprintf("- **Bullet density:** %s\n", patterns.BulletDensity))
	b.WriteString(fmt.Sprintf("- **Heading count:** %d\n", patterns.HeadingCount))
	b.WriteString(fmt.Sprintf("- **Banned word hits:** %d\n", patterns.BannedWordCount))
	if len(patterns.RepeatedOpeners) > 0 {
		b.WriteString(fmt.Sprintf("- **Repeated openers:** %s\n", strings.Join(patterns.RepeatedOpeners, ", ")))
	}
	b.WriteString("\n")

	// Severity counts
	severityCounts := make(map[string]int)
	categoryCounts := make(map[string]int)
	for _, f := range findings {
		severityCounts[f.Severity]++
		categoryCounts[f.Category]++
	}
	b.WriteString("## Summary\n\n")
	b.WriteString(fmt.Sprintf("- **Total findings:** %d\n", len(findings)))
	for sev, count := range severityCounts {
		b.WriteString(fmt.Sprintf("- **%s severity:** %d\n", strings.Title(sev), count))
	}
	b.WriteString("\n")

	if len(findings) == 0 {
		b.WriteString("No slop detected. The text looks clean.\n")
		return b.String()
	}

	// Group by category
	byCategory := make(map[string][]Finding)
	for _, f := range findings {
		byCategory[f.Category] = append(byCategory[f.Category], f)
	}
	var cats []string
	for c := range byCategory {
		cats = append(cats, c)
	}
	sort.Strings(cats)

	for _, cat := range cats {
		b.WriteString(fmt.Sprintf("## %s (%d)\n\n", strings.Title(strings.ReplaceAll(cat, "_", " ")), len(byCategory[cat])))
		for _, f := range byCategory[cat] {
			b.WriteString(fmt.Sprintf("- **Line %d** [%s]\n", f.Line, f.Severity))
			b.WriteString(fmt.Sprintf("  - Text: `%s`\n", f.Text))
			if f.Suggest != "" {
				b.WriteString(fmt.Sprintf("  - Suggest: `%s`\n", f.Suggest))
			}
			b.WriteString(fmt.Sprintf("  - Context: %s\n", f.Context))
			b.WriteString("\n")
		}
	}

	b.WriteString("## Recommended Actions\n\n")
	if patterns.BulletDensity == "high" {
		b.WriteString("- Convert short bullet lists (≤4 items) into flowing prose.\n")
	}
	if patterns.SentenceLengthStdDev < 8 && patterns.AvgSentenceLength > 15 {
		b.WriteString("- Vary sentence length intentionally. Mix short punchy sentences with longer ones.\n")
	}
	if patterns.BannedWordCount > 5 {
		b.WriteString("- Replace banned words with plain alternatives.\n")
	}
	if len(patterns.RepeatedOpeners) > 0 {
		b.WriteString("- Diversify paragraph openers. Drop connectors and start with the subject.\n")
	}
	if categoryCounts["meta_commentary"] > 2 {
		b.WriteString("- Strip meta commentary (\"In this section...\", \"Let's dive in...\").\n")
	}
	if categoryCounts["redefinition"] > 0 {
		b.WriteString("- Remove \"not X, it is Y\" redefinition sentences. Say what it is directly.\n")
	}
	b.WriteString("\n")

	return b.String()
}

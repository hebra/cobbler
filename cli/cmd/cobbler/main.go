package main

import (
	"context"
	"flag"
	"fmt"
	"log"
	"os"
	"sort"
	"strings"
	"text/tabwriter"
	"time"

	"github.com/grandcat/zeroconf"
)

const (
	serviceType   = "_cobbler._tcp"
	serviceDomain = "local."
)

func main() {
	log.SetFlags(0)

	if len(os.Args) < 2 {
		printHelp()
		return
	}

	switch os.Args[1] {
	case "help":
		runHelp(os.Args[2:])
	case "discover":
		if err := runDiscover(os.Args[2:]); err != nil {
			log.Printf("discover: %v", err)
			os.Exit(1)
		}
	default:
		log.Printf("unknown command: %s", os.Args[1])
		fmt.Fprintln(os.Stderr)
		printHelp()
		os.Exit(1)
	}
}

func runHelp(args []string) {
	if len(args) == 0 {
		printHelp()
		return
	}

	switch args[0] {
	case "discover":
		printDiscoverHelp(os.Stdout)
	case "help":
		printHelp()
	default:
		log.Printf("unknown command: %s", args[0])
		fmt.Fprintln(os.Stderr)
		printHelp()
		os.Exit(1)
	}
}

func printHelp() {
	fmt.Println("Usage: cobbler <command> [options]")
	fmt.Println()
	fmt.Println("Commands:")
	fmt.Println("  help [command]  Show help for a command")
	fmt.Println("  discover        Discover cobbler daemons on the local network")
	fmt.Println()
	fmt.Println("Run `cobbler help <command>` for details.")
}

func runDiscover(args []string) error {
	fs := flag.NewFlagSet("discover", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	timeout := fs.Duration("timeout", 3*time.Second, "time to wait for responses")
	fs.Usage = func() {
		printDiscoverHelp(os.Stderr)
	}
	if err := fs.Parse(args); err != nil {
		return err
	}

	resolver, err := zeroconf.NewResolver(nil)
	if err != nil {
		return fmt.Errorf("create resolver: %w", err)
	}

	ctx, cancel := context.WithTimeout(context.Background(), *timeout)
	defer cancel()

	entries := make(chan *zeroconf.ServiceEntry)
	results := make([]*zeroconf.ServiceEntry, 0, 8)
	done := make(chan struct{})

	go func() {
		for entry := range entries {
			results = append(results, entry)
		}
		close(done)
	}()

	if err := resolver.Browse(ctx, serviceType, serviceDomain, entries); err != nil {
		return fmt.Errorf("browse: %w", err)
	}

	<-ctx.Done()
	<-done

	if len(results) == 0 {
		fmt.Println("No cobbler daemons found.")
		return nil
	}

	sort.Slice(results, func(i, j int) bool {
		return results[i].Instance < results[j].Instance
	})

	writer := tabwriter.NewWriter(os.Stdout, 0, 4, 2, ' ', 0)
	fmt.Fprintln(writer, "ID\tHOST\tADDRESS\tPORT\tINSTANCE")
	for _, entry := range results {
		fmt.Fprintf(
			writer,
			"%s\t%s\t%s\t%d\t%s\n",
			entryID(entry),
			strings.TrimSuffix(entry.HostName, "."),
			entryAddresses(entry),
			entry.Port,
			entry.Instance,
		)
	}
	_ = writer.Flush()

	return nil
}

func printDiscoverHelp(out *os.File) {
	fmt.Fprintln(out, "Usage: cobbler discover [options]")
	fmt.Fprintln(out)
	fmt.Fprintf(out, "Discovers services advertised as %s in %s.\n", serviceType, serviceDomain)
	fmt.Fprintln(out)
	fmt.Fprintln(out, "Options:")
	fmt.Fprintln(out, "  -timeout duration   time to wait for responses (default 3s)")
}

func entryID(entry *zeroconf.ServiceEntry) string {
	for _, txt := range entry.Text {
		if strings.HasPrefix(txt, "id=") {
			return strings.TrimPrefix(txt, "id=")
		}
	}
	return ""
}

func entryAddresses(entry *zeroconf.ServiceEntry) string {
	parts := make([]string, 0, len(entry.AddrIPv4)+len(entry.AddrIPv6))
	for _, addr := range entry.AddrIPv4 {
		parts = append(parts, addr.String())
	}
	for _, addr := range entry.AddrIPv6 {
		parts = append(parts, addr.String())
	}
	return strings.Join(parts, ",")
}

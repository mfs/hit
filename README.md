hit
===

## Description

As a Sys Admin I'm constantly checking and adjusting vhosts, redirects, headers,
etc. I normally use `curl -I ...` for this. This works well though has some
issues. It sends a HEAD request which some web servers treat differently to a
GET request. The output can also be quite verbose if there are lots of headers
being returned making it slightly difficult to pick out the information I need.

Hit addresses these issues and is yet another foray into the world of Rust for
me. It performs a GET request for a supplied URL and prints out the headers
returned. Useful information (to me) is highlighted. It looks like this:

![screenshot](https://github.com/mfs/hit/raw/master/screenshots/1.png)

## Todo

 - [ ] The list of headers highlighted should be adjustable at run time.

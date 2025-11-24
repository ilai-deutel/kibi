#!/usr/bin/env Rscript

library(asciicast)
library(httr2)
library(openssl, include.only="base64_encode")
library(xml2, warn.conflicts = FALSE)

args = commandArgs(trailingOnly=TRUE)

# https://fonts.googleapis.com/css2?family=Fira+Code:wght@300..700
font_url <- "https://fonts.gstatic.com/s/firacode/v27/uU9NCBsR6Z2vfE9aq3bh3dSD.woff2"

tmp <- tempfile(fileext = ".svg")
read_cast("assets/recording.cast") |>
write_svg(
  tmp,
  window = TRUE,
  cols = 106,
  theme = "monokai",
  cursor = TRUE,
)
svg <- read_xml(tmp)

font_base64 <- request(font_url) |>
req_perform() |>
resp_body_raw() |>
base64_encode()

css <- sprintf(
  "@font-face {
    font-family: Fira Code;
    src: url('data:font/woff2;base64,%s') format('woff2');
    font-weight: 300 700;
    font-style: normal;
  }",
  font_base64
)

svg <- read_xml(tmp)
svg |> xml_add_child("style", css, type = "text/css")
write_xml(svg, args[1], options="no_declaration")
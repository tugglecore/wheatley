"use client";

import { Box, Text } from "@kuma-ui/core";
import { Crimson_Pro, Lexend, Roboto, Playwrite_US_Modern } from 'next/font/google'

const crimsonPro = Crimson_Pro({
  subsets: ['latin']
})
const lexend = Lexend({ subsets: ['latin'] })
const roboto = Roboto({ subsets: ['latin'], weight: ['400'] })
const playWrite = Playwrite_US_Modern({})

export default function Header() {
  return (
    <Box>
      <Text as="h1">Hey Wheatley</Text>
      <p className={crimsonPro.className}>
        Cool looking font
      </p>
      <p className={lexend.className}>
        Cool looking font
      </p>
      <p className={roboto.className}>
        Cool looking font
      </p>
      <p className={playWrite.className}>
        Cool looking font
      </p>
    </Box>
  );
}

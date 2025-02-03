
"use client";

import { useCallback } from "react";
import { Box, HStack } from "@kuma-ui/core";
import Image from "next/image";
import image1 from "../public/image1.jpg";
import image2 from "../public/image2.jpg";
import image3 from "../public/image3.jpg";

export default function Page() {
  return (
    <div>
      <HStack
        justify="center"
        alignItems="center" 
        gap={7}
        maxWidth="80vw"
        maxHeight="80vh"
        overflow="hidden"
      >
      <Box
        flexBasis='100%'
      >
        <Image 
          alt=".." 
          src={image1}
          style={{
            width: '100%',
            height: 'auto'
          }}
        />
      </Box>
      <Box
        flexBasis='100%'
      >
        <Image 
          alt=".." 
          src={image2}
          style={{
            width: '100%',
            height: 'auto'
          }}
        />
      </Box>
      <Box
        flexBasis='100%'
      >
        <Image 
          alt=".." 
          src={image3}
          style={{
            width: '100%',
            height: 'auto'
          }}
        />
      </Box>
      </HStack>
    </div>
  );
}

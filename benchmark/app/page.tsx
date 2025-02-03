"use client";

import { useCallback } from "react";
import Link from 'next/link'
import { HStack } from "@kuma-ui/core";
import { VisXYContainer, VisLine, VisAxis } from "@unovis/react";
import "chart.js/auto";
import { Doughnut } from "react-chartjs-2";
import { VictoryChart, VictoryGroup, VictoryTheme, VictoryBar } from "victory";
import { Crimson_Pro, Lexend, Roboto, Playwrite_US_Modern } from 'next/font/google'
import Images from './images'
import Header from './header'

const crimsonPro = Crimson_Pro({
  subsets: ['latin']
})
const lexend = Lexend({ subsets: ['latin'] })
const roboto = Roboto({ subsets: ['latin'], weight: ['400'] })
const playWrite = Playwrite_US_Modern({})

function BasicBar() {
  return (
    <div style={{ width: "80vw", height: "80vh" }}>
      <VictoryChart
        theme={VictoryTheme.clean}
        domain={{ y: [0.5, 5.5] }}
        domainPadding={{ x: 40 }}
      >
        <VictoryGroup offset={20} style={{ data: { width: 15 } }}>
          <VictoryBar
            data={[
              { x: "2023 Q1", y: 1 },
              { x: "2023 Q2", y: 2 },
              { x: "2023 Q3", y: 3 },
              { x: "2023 Q4", y: 2 },
            ]}
          />
          <VictoryBar
            data={[
              { x: "2023 Q1", y: 2 },
              { x: "2023 Q2", y: 3 },
              { x: "2023 Q3", y: 4 },
              { x: "2023 Q4", y: 5 },
            ]}
          />
          <VictoryBar
            data={[
              { x: "2023 Q1", y: 1 },
              { x: "2023 Q2", y: 2 },
              { x: "2023 Q3", y: 3 },
              { x: "2023 Q4", y: 4 },
            ]}
          />
        </VictoryGroup>
      </VictoryChart>
    </div>
  );
}

type DataRecord = { x: number; y: number };

const data: DataRecord[] = [
  { x: 0, y: 0 },
  { x: 1, y: 2 },
  { x: 2, y: 1 },
];

function BasicDoughnut() {
  const data = {
    labels: ["Red", "Blue", "Yellow"],
    datasets: [
      {
        label: "My First Dataset",
        data: [300, 50, 100],
        backgroundColor: [
          "rgb(255, 99, 132)",
          "rgb(54, 162, 235)",
          "rgb(255, 205, 86)",
        ],
        hoverOffset: 4,
      },
    ],
  };
  return (
    <div style={{ width: "80vw", height: "80vh" }}>
      <Doughnut data={data} />
    </div>
  );
}

function BasicLineChart() {
  return (
    <div style={{ width: "80vw", height: "80vh" }}>
      <VisXYContainer data={data}>
        <VisLine<DataRecord>
          x={useCallback((d) => d.x, [])}
          y={useCallback((d) => d.y, [])}
        ></VisLine>
        <VisAxis type="x"></VisAxis>
        <VisAxis type="y"></VisAxis>
      </VisXYContainer>
    </div>
  );
}

export default function Page() {
  return (
    <div>
      <Header />
      <BasicLineChart />
      <BasicDoughnut />
      <BasicBar />
      <Images />
    </div>
  );
}

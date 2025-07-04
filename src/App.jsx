import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import ParseCsv from "./pages/ParseCsv";

function App() {


  return (
    <main className="container">
      <h1>اپنے نامہ امل فراہم کریں۔</h1>

      <ParseCsv/>
    </main>
  );
}

export default App;

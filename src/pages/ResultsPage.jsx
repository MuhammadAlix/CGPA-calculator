import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

function ResultPage() {
  const [regNumber, setRegNumber] = useState("");
  const [result, setResult] = useState(null);
  const [loading, setLoading] = useState(false);
  const [errorMsg, setErrorMsg] = useState("");

  const fetchResults = async () => {
    setLoading(true);
    setErrorMsg("");
    try {
      const res = await invoke("process_student_results", {
        regNumber: regNumber.trim(),
      });
      setResult(res);
    } catch (err) {
      console.error(err);
      setErrorMsg("Failed to fetch results. Please check the registration number.");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{ padding: "20px", fontFamily: "Arial" }}>
      <h1>📘 GPA & CGPA Calculator</h1>
      <input
        type="text"
        value={regNumber}
        onChange={(e) => setRegNumber(e.target.value)}
        placeholder="Enter Registration Number"
        style={{ padding: "10px", width: "300px", fontSize: "16px" }}
      />
      <br />
      <button
        onClick={fetchResults}
        disabled={!regNumber.trim() || loading}
        style={{
          marginTop: "10px",
          padding: "10px 20px",
          fontSize: "16px",
          cursor: "pointer",
        }}
      >
        {loading ? "Fetching..." : "Get Results"}
      </button>

      {errorMsg && (
        <div style={{ color: "red", marginTop: "10px" }}>{errorMsg}</div>
      )}

      {result && (
        <div style={{ marginTop: "20px" }}>
          <h2>🎓 Final CGPA: {result.cgpa} / 4.0</h2>
          <h3>📚 Semester-wise GPA:</h3>
          <ul>
            {result.semesters.map((sem, idx) => (
              <li key={idx}>
                {sem.semester_label}: GPA = {sem.gpa}
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}

export default ResultPage;
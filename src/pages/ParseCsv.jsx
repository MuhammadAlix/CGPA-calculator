import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

function CgpaCalculator() {
  const [semesters, setSemesters] = useState([
    { name: "Semester 1", subjects: [{ ch: '', tm: '', om: '' }] }
  ]);
  const [result, setResult] = useState('');
  const [status, setStatus] = useState('');

  const addSemester = () => {
    setSemesters([...semesters, {
      name: `Semester ${semesters.length + 1}`,
      subjects: [{ ch: '', tm: '', om: '' }]
    }]);
  };

  const addSubject = (semIndex) => {
    const updated = [...semesters];
    updated[semIndex].subjects.push({ ch: '', tm: '', om: '' });
    setSemesters(updated);
  };

  const removeSubject = (semIndex, subIndex) => {
    const updated = [...semesters];
    updated[semIndex].subjects.splice(subIndex, 1);
    setSemesters(updated);
  };

  const updateField = (semIndex, subIndex, field, value) => {
    const updated = [...semesters];
    updated[semIndex].subjects[subIndex][field] = value;
    setSemesters(updated);
  };

const handleSubmit = async () => {
  setStatus("⏳ Generating CSV data...");

  // Create CSV rows: each subject on a new line
  const csvRows = [];

  // Optional: add a header or semester titles
  csvRows.push("CH,TM,OM,Semester"); // optional header

  semesters.forEach((sem, semIndex) => {
    sem.subjects.forEach((sub) => {
      const line = [
        sub.ch.toString().trim(),
        sub.tm.toString().trim(),
        sub.om.toString().trim(),
        `Semester ${semIndex + 1}`,
      ].join(",");
      csvRows.push(line);
    });
  });

  const csvString = csvRows.join("\n");

  console.log("📤 Sending CSV string to backend:\n", csvString);

  try {
    const response = await invoke("process_multiseme_csv_data", {
      csvString: csvString,
    });
    setResult(response);
    setStatus("✅ Done!");
  } catch (err) {
    console.error("❌ Failed to process:", err);
    setStatus("❌ Failed");
    setResult("");
  }
};


  return (
    <div style={{ padding: '20px' }}>
      <h2>CGPA Calculator</h2>

      {semesters.map((sem, sIdx) => (
        <div key={sIdx} style={{ marginBottom: '30px', borderBottom: '1px solid #ccc' }}>
          <h3>{sem.name}</h3>

          <table border="1" cellPadding="4" style={{ width: '100%', marginBottom: '10px' }}>
            <thead>
              <tr>
                <th>Credit Hours</th>
                <th>Total Marks</th>
                <th>Obtained Marks</th>
                <th>Action</th>
              </tr>
            </thead>
            <tbody>
              {sem.subjects.map((sub, subIdx) => (
                <tr key={subIdx}>
                  <td>
                    <input
                      type="number"
                      value={sub.ch}
                      onChange={e => updateField(sIdx, subIdx, 'ch', e.target.value)}
                    />
                  </td>
                  <td>
                    <input
                      type="number"
                      value={sub.tm}
                      onChange={e => updateField(sIdx, subIdx, 'tm', e.target.value)}
                    />
                  </td>
                  <td>
                    <input
                      type="number"
                      value={sub.om}
                      onChange={e => updateField(sIdx, subIdx, 'om', e.target.value)}
                    />
                  </td>
                  <td>
                    {sem.subjects.length > 1 && (
                      <button onClick={() => removeSubject(sIdx, subIdx)}>🗑</button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>

          <button onClick={() => addSubject(sIdx)}>➕ Add Subject</button>
        </div>
      ))}

      <button onClick={addSemester} style={{ marginRight: '10px' }}>➕ Add Semester</button>
      <button onClick={handleSubmit}>✅ Calculate CGPA</button>

      <p>{status}</p>
      {result && <pre style={{ padding: '10px' }}>{result}</pre>}
    </div>
  );
}

export default CgpaCalculator;

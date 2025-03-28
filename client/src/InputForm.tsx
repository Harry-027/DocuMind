import { useState } from 'react';
import { invoke } from "@tauri-apps/api/core";
import { Doc } from './DataModel';

interface InputFormProps {
    selectedItem: Doc | null;
  }

const InputForm : React.FC<InputFormProps> = ({ selectedItem }) => {
  const [input, setInput] = useState<string>('');
  const [llmResponse, setLLmResponse] = useState<string>('');

  const handleSubmit = async () => {
    setLLmResponse('Loading...');
    const response = await invoke<string>('process_prompt', { item: selectedItem, query: input });
    setLLmResponse(response);
  };

  const handleKeyDown = (e: any) => {
    if (e.key === 'Enter') {
      handleSubmit();
    }
  };

  return (
    <div className="flex-4 flex-col">
      <hr />
      <div className="flex items-center gap-25 mt-8">
      <p className="text-lg font-bold">Query::</p>
      <input
        type="text"
        value={input}
        onChange={(e) => setInput(e.target.value)}
        onKeyDown={handleKeyDown}
        className="w-[70%] p-2 rounded-2xl text-lg placeholder-gray-400 outline-double outline-2 outline-offset-2" placeholder="Prompt here..."/>
      
      <button onClick={handleSubmit} className="font-bold cursor-pointer px-5 outline-none outline-2 outline-offset-2">
      <svg xmlns="http://www.w3.org/2000/svg" className="h-12 w-12" viewBox="0 0 24 24" fill="none" stroke="currentColor">
         <path d="M5 12h14" />
         <path d="M12 5l7 7-7 7" />
      </svg>Submit</button>
      </div>
      <div className="flex items-center gap-4 mt-8">
      <p className="w-[10%] text-lg font-bold leading-relaxed">LLM Response::</p>
      <p className="text-lg leading-relaxed tracking-wide w-[90%] overflow-y-auto max-h-40">{llmResponse}</p>
      </div>
    </div>
  );
};

export default InputForm;
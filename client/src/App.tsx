import "./App.css";

import { useState, useEffect } from 'react';
import { invoke } from "@tauri-apps/api/core";

import Sidebar from './Sidebar';
import Content from './Content';
import InputForm from './InputForm';
import AddButton from './AddButton';
import Modal from './Modal';
import { Doc } from './DataModel';


function App() {
  const [items, setItems] = useState<Doc[]>([]);
  const [selectedItem, setSelectedItem] = useState<Doc | null>(null);
  const [isModalOpen, setModalOpen] = useState(false);
  const [status, setStatus] = useState(false);

  useEffect(() => {
    fetchItems();
  }, []);

  const fetchItems = async () => {
    try {
      const response = await invoke<Doc[]>('fetch_list_items');
      setItems(response);
    } catch(error){
      console.log("Error fetching items", error);
    }
  };

  const handleAddClick = () => {
    setModalOpen(true);
  };

  const handleSubmit = async (file: File | null ) => {
    if (!file) {
      return;
    }
    const reader = new FileReader();
    reader.readAsDataURL(file);

    reader.onload = async () => {
      if (!reader.result) {
        console.error('FileReader result is null');
        return;
      } else {
        setStatus(true);
        const resultString = reader.result?.toString();
        if (resultString && resultString.startsWith('data:')) {
          const base64Data = resultString.split(',')[1];
          try {
            console.log('Uploading file:', file.name);
            console.log('Uploading base64Data:', base64Data);
            const result = await invoke<string>('upload_file', {name: file.name,ct: base64Data});
            console.log('Upload successful:', result);
            fetchItems();
          } catch (error) {
            console.error('Upload failed:', error);
          }
        } else {
            console.error('Unexpected file format');
        }
        setStatus(false);
      }
    }
  };

  return (
    <div>
      <AddButton onClick={handleAddClick} status={status} />
      <div className="flex h-screen gap-6 w-full overflow-hidden text-white">
      <Sidebar items={items} onSelect={setSelectedItem} />
      <div className="flex-2 flex flex-col">
        {selectedItem ? <Content selectedItem={selectedItem} /> : <div className="flex-1 flex flex-col text-center font-bold p-50">Select document from sidebar...</div>}
        {selectedItem && <InputForm selectedItem={selectedItem} />}
      </div>
      <Modal isOpen={isModalOpen} onClose={() => setModalOpen(false)} onSubmit={handleSubmit} />
    </div>
    </div>
  );
}

export default App;

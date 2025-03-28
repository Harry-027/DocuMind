import { useState, useEffect } from 'react';
import { invoke } from "@tauri-apps/api/core";
import { Doc } from './DataModel';

const Content = ({ selectedItem }: { selectedItem: Doc }) => {
  const [content, setContent] = useState<string>('');

  useEffect(() => {
    fetchContent();
  }, [selectedItem]);

  const fetchContent = async () => {
    setContent('Loading...');
    const response = await invoke<string>('fetch_content', { item: selectedItem });
    setContent(response);
  };
  return (
    <div className="flex-1 p-4">
      <h2 className="text-2xl font-bold mb-4 text-center">{selectedItem.name}</h2>
      <h4 className="text-2xl font-bold mb-2 mt-12">Uploaded raw content--</h4>
      <hr/>
      <div className="w-[85vw] h-[50vh] overflow-y-auto overflow-x-auto p-4 bg-gray-100 border rounded-lg" dangerouslySetInnerHTML={{ __html: content }}>
      </div>
    </div>
  );
};

export default Content;

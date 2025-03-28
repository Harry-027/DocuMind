import React, { useState } from 'react';
import { X } from 'lucide-react';

const Modal = ({ isOpen, onClose, onSubmit }: any) => {
  const [file, setFile] = useState<File | null>(null);

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files.length > 0) {
      setFile(e.target.files[0]);
    }
  };

  const handleSubmit = () => {
    if(file) {
      onSubmit(file);
    } 
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center">
      <div className="bg-white p-6 rounded-lg shadow-lg w-96 text-black">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-xl font-bold">Add Item</h2>
          <button onClick={onClose}>
            <X className="w-5 h-5" />
          </button>
        </div>
        <label className="block mb-4 text-gray italic">
          Upload File:
          <input
            type="file"
            className="w-full p-2 border rounded mt-1 outline-dashed outline-2 outline-offset-2"
            onChange={handleFileChange}
          />
        </label>

        <button
          className="w-full py-2 cursor-pointer hover:bg-red-900"
          onClick={handleSubmit}
        >
          Submit
        </button>
      </div>
    </div>
  );
};

export default Modal;

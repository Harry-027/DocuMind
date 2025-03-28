import { Plus } from 'lucide-react';

interface AddButtonProps {
  onClick?: () => void;
  status?: boolean; // New property
}

const AddButton = ({ onClick, status }: AddButtonProps) => {
  return (
    status ? <p className="text-white font-bold px-8"> Uploading... </p> : (
    <button
      onClick={onClick}
      className="flex items-center px-4 py-2 mt-12 bg-blue-500 hover:bg-blue-600 text-white font-semibold rounded-lg shadow-md focus:outline-none focus:ring-2 focus:ring-blue-400 focus:ring-opacity-75">
      <Plus className="w-5 h-5 mr-2" />
      Upload Document
    </button>
  ));
};

export default AddButton;

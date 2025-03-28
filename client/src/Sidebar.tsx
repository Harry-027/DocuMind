import { Doc } from "./DataModel";

const Sidebar = ({ items, onSelect }: { items: Doc[]; onSelect: (item: Doc | null) => void }) => {
  return (
    <div className="h-screen w-50 p-5 flex flex-col mt-19">
      <h2 className="text-2xl font-bold tracking-wide mb-2">Documents</h2>
      <hr/>
      <ul>
        {items.map((item, index) => (
          <li key={index} className="overflow-hidden cursor-pointer py-2 hover:bg-gray-600 items-center" onClick={() => onSelect(item)}>
            {item.name}
          </li>
        ))}
      </ul>
    </div>
  );
};

export default Sidebar;

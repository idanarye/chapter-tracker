(ns chapter-tracker.view.series-directories-panel (:gen-class)
  (:use [chapter-tracker.view tools create-directory-dialog])
  (:use chapter-tracker.model)
)
(import
  '(java.awt GridBagConstraints Dimension)
  '(javax.swing JPanel JTable JScrollPane JLabel)
  '(javax.swing.table DefaultTableModel TableCellRenderer)
  '(javax.swing.event ListSelectionListener)
)

(def directory-table-columns [
                              {:caption "Id"             :field :directory-id     :editable false}
                              {:caption "Directory"      :field :directory        :editable true}
                              {:caption "Pattern"        :field :pattern          :editable true}
                             ])

(def directory-table-captions (to-array (map :caption directory-table-columns)))

(defn load-series-directories-to-table [table series-record]
  (..
    table
    getModel
    (setDataVector
      (to-array-2d
        (map (fn [directory]
               (to-array (map #(% directory) (map :field directory-table-columns)))
             )
             (fetch-directory-records-for series-record)
             ))
      directory-table-captions
    )
  )
)

(defn create-series-directories-panel[]
  (let [series-record-atom (atom nil)
        series-name-label     (JLabel.)
        directories-table     (JTable. (proxy [DefaultTableModel] []
                                         (isCellEditable [row column] false)
                                       ))
        create-directory-button  (action-button "Create Directory"
                                                (if @series-record-atom
                                                  (.show (create-create-directory-dialog @series-record-atom
                                                                                         #(load-series-directories-to-table directories-table
                                                                                                                            @series-record-atom)
                                                         ))))
        delete-directory-button  (action-button "Delete Directory"
                                                (delete-directory-record (.. directories-table getModel (getValueAt (.getSelectedRow directories-table) 0)))
                                                (load-series-directories-to-table directories-table @series-record-atom)
                                 )
        directories-scroll-pane (JScrollPane. directories-table)
       ]
    [(create-panel {:width 200 :height 150} ;series-directories panel

                   (add-with-constraints create-directory-button
                                         (gridx 0) (gridy 0) (fill GridBagConstraints/BOTH))
                   (add-with-constraints series-name-label
                                         (gridx 1) (gridy 0) (fill GridBagConstraints/BOTH))
                   (add-with-constraints delete-directory-button
                                         (gridx 2) (gridy 0) (fill GridBagConstraints/BOTH))

                   (.. directories-table getModel (setColumnIdentifiers directory-table-captions))
                   (.setPreferredSize directories-scroll-pane (Dimension. 140 200))
                   (add-with-constraints directories-scroll-pane
                                         (gridx 0) (gridy 1) (gridwidth 3) (fill GridBagConstraints/BOTH))
     ) (fn [series-record] ;updating function
         (reset! series-record-atom series-record)
         (.setText series-name-label (->> (or series-record "") .toString (take 15) (apply str)))
         (load-series-directories-to-table directories-table series-record)
       )]
  )
)
